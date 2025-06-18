use crate::env_variables::EnvVariables;
use crate::errors::{Error, Result};
use crate::utils::pay_invoices;
use anyhow::anyhow;
use itertools::Itertools;
use node::{MeltRequest, MintQuoteState, NodeClient, QuoteStateRequest, hash_melt_request};
use primitive_types::U256;
use rusqlite::Connection;
use starknet_types::{Asset, STARKNET_STR, Unit};
use starknet_types_core::felt::Felt;
use std::time::Duration;
use tonic::transport::Channel;
use wallet::types::NodeUrl;
use wallet::{
    self,
    types::compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
};

pub struct WalletOps {
    db_conn: Connection,
    node_id: u32,
    node_client: NodeClient<Channel>,
}

impl WalletOps {
    pub fn new(db_conn: Connection, node_id: u32, node_client: NodeClient<Channel>) -> Self {
        WalletOps {
            db_conn,
            node_id,
            node_client,
        }
    }

    pub async fn mint(&mut self, amount: U256, asset: Asset, env: EnvVariables) -> Result<()> {
        let amount = amount
            .checked_mul(asset.precision())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _remainder) = asset
            .convert_to_amount_and_unit(amount)
            .map_err(|e| Error::Other(e.into()))?;

        let tx = self.db_conn.transaction()?;
        let quote = wallet::create_mint_quote(
            &tx,
            &mut self.node_client,
            self.node_id,
            STARKNET_STR.to_string(),
            amount,
            unit.as_str(),
        )
        .await?;
        tx.commit()?;

        let calls: [starknet_types::Call; 2] = serde_json::from_str(&quote.request)?;
        pay_invoices(calls.to_vec(), env).await?;

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let state = match wallet::get_mint_quote_state(
                &self.db_conn,
                &mut self.node_client,
                STARKNET_STR.to_string(),
                quote.quote.clone(),
            )
            .await?
            {
                Some(s) => s,
                None => {
                    println!("quote {} has expired", quote.quote);
                    return Ok(());
                }
            };
            if state == MintQuoteState::MnqsPaid {
                break;
            }
        }

        let tx = self.db_conn.transaction()?;
        wallet::mint_and_store_new_tokens(
            &tx,
            &mut self.node_client,
            STARKNET_STR.to_string(),
            quote.quote,
            self.node_id,
            unit.as_str(),
            amount,
        )
        .await?;
        tx.commit()?;
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
            .checked_mul(asset.precision())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _) = asset
            .convert_to_amount_and_unit(amount)
            .map_err(|e| Error::Other(e.into()))?;
        let tx = self.db_conn.transaction()?;
        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            &tx,
            &mut self.node_client,
            self.node_id,
            amount,
            unit.as_str(),
        )
        .await?
        .ok_or(anyhow!("not enough funds"))?;
        tx.commit()?;

        let tx = self.db_conn.transaction()?;
        let proofs = wallet::load_tokens_from_db(&tx, proofs_ids).await?;
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
        let wad = CompactWad {
            node_url,
            unit,
            memo,
            proofs: compact_proofs,
        };
        tx.commit()?;
        // println!("{}", wad.to_string());
        Ok(wad)
    }

    pub async fn receive(&mut self, wad: &CompactWad<Unit>) -> Result<()> {
        wallet::receive_wad(
            &mut self.db_conn,
            &mut self.node_client,
            self.node_id,
            wad.unit.as_str(),
            &wad.proofs,
        )
        .await?;
        Ok(())
    }

    pub async fn melt(&mut self, amount: U256, asset: Asset, to: String) -> Result<()> {
        let amount = amount
            .checked_mul(asset.precision())
            .ok_or(anyhow!("amount too big"))?;
        let (amount, unit, _) = asset
            .convert_to_amount_and_unit(amount)
            .map_err(|e| Error::Other(e.into()))?;

        let tx = self.db_conn.transaction()?;
        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            &tx,
            &mut self.node_client,
            self.node_id,
            amount,
            unit.as_str(),
        )
        .await?
        .ok_or(anyhow!("not enough funds"))?;
        tx.commit()?;

        let tx = self.db_conn.transaction()?;
        let inputs = wallet::load_tokens_from_db(&tx, proofs_ids).await?;
        let payee_address = Felt::from_hex(&to).map_err(|e| Error::Other(e.into()))?;
        if !starknet_types::is_valid_starknet_address(&payee_address) {
            return Err(Error::Other(anyhow!(
                "Invalid starknet address: {}",
                payee_address
            )));
        }
        let melt_request = MeltRequest {
            method: STARKNET_STR.to_string(),
            unit: unit.to_string(),
            request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                payee: payee_address,
                asset: starknet_types::Asset::Strk,
            })?,
            inputs: wallet::convert_inputs(&inputs),
        };
        let melt_request_hash = hash_melt_request(&melt_request);
        let resp = self.node_client.melt(melt_request).await?.into_inner();
        wallet::db::register_melt_quote(&tx, self.node_id, &resp)?;
        tx.commit()?;
        wallet::acknowledge(
            &mut self.node_client,
            nuts::nut19::Route::Melt,
            melt_request_hash,
        )
        .await?;

        loop {
            let melt_quote_state_response = self
                .node_client
                .melt_quote_state(QuoteStateRequest {
                    method: "starknet".to_string(),
                    quote: resp.quote.clone(),
                })
                .await?
                .into_inner();

            if !melt_quote_state_response.transfer_ids.is_empty() {
                println!(
                    "{}",
                    WalletOps::format_melt_transfers_id_into_term_message(
                        melt_quote_state_response.transfer_ids
                    )
                );
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        Ok(())
    }

    fn format_melt_transfers_id_into_term_message(transfer_ids: Vec<String>) -> String {
        let mut string_to_print = "Melt done. Withdrawal settled with tx".to_string();
        if transfer_ids.len() != 1 {
            string_to_print.push('s');
        }
        string_to_print.push_str(": ");
        let mut iterator = transfer_ids.into_iter();
        string_to_print.push_str(&iterator.next().unwrap());
        for tx_hash in iterator {
            string_to_print.push_str(", ");
            string_to_print.push_str(&tx_hash);
        }

        string_to_print
    }
}
