use anyhow::{Result, anyhow};
use bip39::Mnemonic;
use itertools::Itertools;
use node_client::NodeClient;
use nuts::{Amount, nut01::PublicKey};
use primitive_types::U256;
use r2d2_sqlite::SqliteConnectionManager;
use starknet_types::{Asset, DepositPayload, STARKNET_STR, constants::ON_CHAIN_CONSTANTS};
use starknet_types_core::felt::Felt;
use tonic::transport::Channel;
use wallet::{
    self,
    db::{balance::Balance, wad::delete_wad},
    types::{
        NodeUrl,
        compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
    },
};

use crate::common::utils::{EnvVariables, starknet::pay_invoices};

type Pool = r2d2::Pool<SqliteConnectionManager>;
pub struct WalletOps {
    db_pool: Pool,
    node_id: u32,
    node_client: NodeClient<Channel>,
}

impl WalletOps {
    pub fn new(db_pool: Pool, node_id: u32, node_client: NodeClient<Channel>) -> Self {
        WalletOps {
            db_pool,
            node_id,
            node_client,
        }
    }

    pub fn init(&self) -> Result<Mnemonic> {
        let seed_phrase = wallet::seed_phrase::create_random()?;
        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;

        let db_conn = &*self.db_pool.get()?;
        wallet::wallet::init(seed_phrase_manager, db_conn, &seed_phrase)?;

        Ok(seed_phrase)
    }

    pub async fn restore(&self, seed_phrase: Mnemonic) -> Result<()> {
        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;
        {
            let db_conn = &*self.db_pool.get()?;
            wallet::wallet::restore(seed_phrase_manager.clone(), db_conn, seed_phrase)?;
        };

        wallet::node::restore(
            seed_phrase_manager,
            self.db_pool.clone(),
            self.node_id,
            self.node_client.clone(),
        )
        .await?;

        Ok(())
    }

    pub fn balance(&self) -> Result<Vec<Balance>> {
        let db_conn = &*self.db_pool.get()?;
        let balances = wallet::db::balance::get_for_node(db_conn, self.node_id)?;

        Ok(balances)
    }

    pub async fn mint(&mut self, amount: U256, asset: Asset, env: EnvVariables) -> Result<()> {
        let amount = amount
            .checked_mul(asset.scale_factor())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

        let quote = wallet::mint::create_quote(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            STARKNET_STR.to_string(),
            amount,
            unit,
        )
        .await?;

        let on_chain_constants = ON_CHAIN_CONSTANTS.get(env.chain_id.as_str()).unwrap();
        let deposit_payload: DepositPayload = serde_json::from_str(&quote.request)?;
        pay_invoices(
            deposit_payload
                .call_data
                .to_starknet_calls(on_chain_constants.invoice_payment_contract_address)
                .to_vec(),
            env,
        )
        .await?;

        match wallet::mint::wait_for_quote_payment(
            self.db_pool.clone(),
            &mut self.node_client,
            STARKNET_STR.to_string(),
            quote.quote.clone(),
        )
        .await?
        {
            wallet::mint::QuotePaymentIssue::Expired => {
                println!("quote {} has expired", quote.quote);
                return Ok(());
            }
            wallet::mint::QuotePaymentIssue::Paid => {}
        }

        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;
        wallet::mint::redeem_quote(
            seed_phrase_manager,
            self.db_pool.clone(),
            &mut self.node_client,
            STARKNET_STR.to_string(),
            &quote.quote,
            self.node_id,
            unit.as_str(),
            amount,
        )
        .await?;

        Ok(())
    }

    pub async fn send(
        &mut self,
        node_url: NodeUrl,
        amount: U256,
        asset: Asset,
        memo: Option<String>,
    ) -> Result<CompactWad> {
        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;
        let amount = amount
            .checked_mul(asset.scale_factor())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _) = asset.convert_to_amount_and_unit(amount)?;
        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            seed_phrase_manager,
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            amount,
            unit.as_str(),
        )
        .await?
        .ok_or(anyhow!("not enough funds"))?;

        let mut db_conn = self.db_pool.get()?;
        let tx = db_conn.transaction()?;
        let proofs = wallet::unprotected_load_tokens_from_db(&tx, &proofs_ids)?;
        let compact_proofs = proofs
            .into_iter()
            .chunk_by(|p| p.keyset_id)
            .into_iter()
            .map(|(keyset_id, proofs)| CompactKeysetProofs {
                keyset_id,
                proofs: proofs
                    .map(|p| CompactProof {
                        amount: p.amount,
                        secret: p.secret,
                        c: p.c,
                    })
                    .collect(),
            })
            .collect();

        wallet::db::wad::register_wad(
            &tx,
            wallet::db::wad::WadType::OUT,
            &node_url,
            &None,
            &proofs_ids,
        )?;
        tx.commit()?;

        Ok(CompactWad {
            node_url,
            unit: unit.to_string(),
            memo,
            proofs: compact_proofs,
        })
    }

    pub async fn receive(&mut self, wad: &CompactWad) -> Result<()> {
        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;
        wallet::receive_wad(
            seed_phrase_manager,
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            &wad.node_url,
            wad.unit.as_str(),
            wad.proofs.clone(),
            wad.memo(),
        )
        .await?;

        Ok(())
    }

    pub async fn melt(&mut self, amount: U256, asset: Asset, to: String) -> Result<()> {
        let method = STARKNET_STR.to_string();
        let payee_address = Felt::from_hex(&to)?;
        if !starknet_types::is_valid_starknet_address(&payee_address) {
            return Err(anyhow!("Invalid starknet address: {}", payee_address));
        }

        let amount = amount
            .checked_mul(asset.scale_factor())
            .ok_or(anyhow!("amount too big"))?;
        let request = serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
            payee: payee_address,
            asset: starknet_types::Asset::Strk,
            amount: amount.into(),
        })?;

        let unit = asset.find_best_unit();

        let melt_quote_response = wallet::melt::create_quote(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            method.clone(),
            unit.to_string(),
            request,
        )
        .await?;

        let seed_phrase_manager =
            wallet::wallet::sqlite::SeedPhraseManager::new(self.db_pool.clone())?;
        let _melt_response = wallet::melt::pay_quote(
            seed_phrase_manager,
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            melt_quote_response.quote.clone(),
            Amount::from(melt_quote_response.amount),
            method.clone(),
            unit.as_str(),
        )
        .await?;

        if wallet::melt::wait_for_payment(
            self.db_pool.clone(),
            &mut self.node_client,
            method,
            melt_quote_response.quote,
        )
        .await?
        .is_none()
        {
            panic!("quote expired")
        }

        Ok(())
    }

    pub async fn sync_wads(&mut self) -> Result<()> {
        wallet::sync::pending_wads(self.db_pool.clone(), None).await?;

        Ok(())
    }
}

pub async fn recieve_already_spent_wad(wallet_ops: &mut WalletOps, wad: &CompactWad) -> Result<()> {
    let db_conn = wallet_ops.db_pool.get()?;
    let proof_ids = wad
        .proofs()
        .iter()
        .map(|p| p.y().unwrap())
        .collect::<Vec<PublicKey>>();
    let proofs_state = wallet::db::proof::get_proofs_by_ids(&db_conn, &proof_ids)?;
    assert_eq!(proof_ids.len(), proofs_state.len());

    wallet::db::proof::delete_proofs(&db_conn, &proof_ids)?;
    delete_wad(&db_conn, &wad.node_url, &proof_ids)?;

    match wallet_ops.receive(wad).await {
        Err(e) => eprintln!("Recieve Error: {e:?}"),
        Ok(_) => panic!("Double spend should have failed"),
    }

    let proofs_state = wallet::db::proof::get_proofs_state_by_ids(&db_conn, &proof_ids)?;
    assert_eq!(proof_ids.len(), proofs_state.len());
    Ok(())
}
