use anyhow::anyhow;
use itertools::Itertools;
use node_client::NodeClient;
use nuts::Amount;
use primitive_types::U256;
use r2d2_sqlite::SqliteConnectionManager;
use starknet_types::{Asset, STARKNET_STR, Unit};
use starknet_types_core::felt::Felt;
use tonic::transport::Channel;
use wallet::{
    self,
    types::{
        NodeUrl,
        compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
    },
};

use crate::common::{
    error::{Error, Result},
    utils::{EnvVariables, starknet::pay_invoices},
};

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

    pub async fn mint(&mut self, amount: U256, asset: Asset, env: EnvVariables) -> Result<()> {
        let amount = amount
            .checked_mul(asset.scale_factor())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _remainder) = asset
            .convert_to_amount_and_unit(amount)
            .map_err(|e| Error::Other(e.into()))?;

        let quote = wallet::mint::create_quote(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            STARKNET_STR.to_string(),
            amount,
            unit,
        )
        .await?;

        let calls: [starknet_types::Call; 2] = serde_json::from_str(&quote.request)?;
        pay_invoices(calls.to_vec(), env).await?;

        match wallet::mint::wait_for_quote_payment(
            &*self.db_pool.get()?,
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

        wallet::mint::redeem_quote(
            self.db_pool.clone(),
            &mut self.node_client,
            STARKNET_STR.to_string(),
            quote.quote,
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
    ) -> Result<CompactWad<Unit>> {
        let amount = amount
            .checked_mul(asset.scale_factor())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _) = asset
            .convert_to_amount_and_unit(amount)
            .map_err(|e| Error::Other(e.into()))?;
        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            amount,
            unit,
        )
        .await?
        .ok_or(anyhow!("not enough funds"))?;

        let proofs = wallet::load_tokens_from_db(&*self.db_pool.get()?, &proofs_ids)?;
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

        Ok(CompactWad {
            node_url,
            unit,
            memo,
            proofs: compact_proofs,
        })
    }

    pub async fn receive(&mut self, wad: &CompactWad<Unit>) -> Result<()> {
        wallet::receive_wad(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            wad.unit.as_str(),
            wad.proofs.clone(),
        )
        .await?;
        Ok(())
    }

    pub async fn melt(&mut self, amount: U256, asset: Asset, to: String) -> Result<()> {
        let method = STARKNET_STR.to_string();
        let payee_address = Felt::from_hex(&to).map_err(|e| Error::Other(e.into()))?;
        if !starknet_types::is_valid_starknet_address(&payee_address) {
            return Err(Error::Other(anyhow!(
                "Invalid starknet address: {}",
                payee_address
            )));
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
            unit,
            request,
        )
        .await?;

        let _melt_response = wallet::melt::pay_quote(
            self.db_pool.clone(),
            &mut self.node_client,
            self.node_id,
            melt_quote_response.quote.clone(),
            Amount::from(melt_quote_response.amount),
            method.clone(),
            unit,
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
}
