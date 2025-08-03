use std::{cmp::Ordering, str::FromStr};

use nuts::Amount;
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError};
use tauri::{AppHandle, Emitter, State};
use wallet::types::compact_wad::CompactWads;

use crate::{
    AppState,
    commands::BalanceChange,
    parse_asset_amount::{ParseAmountStringError, parse_asset_amount},
};

#[derive(Debug, thiserror::Error)]
pub enum CreateWadsError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error(transparent)]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error(transparent)]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("not enought funds, asked {0}, missing {1}")]
    NotEnoughFunds(Amount, Amount),
    #[error("not enought funds in node {0}")]
    NotEnoughFundsInNode(u32),
    #[error(transparent)]
    NodeConnect(#[from] wallet::ConnectToNodeError),
}

impl serde::Serialize for CreateWadsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn create_wads(
    app: AppHandle,
    state: State<'_, AppState>,
    amount: String,
    asset: String,
) -> Result<String, CreateWadsError> {
    let asset = Asset::from_str(&asset)?;
    let unit = asset.find_best_unit();
    let amount = parse_asset_amount(&amount, asset, unit)?;

    let amount_to_use_per_node = {
        let db_conn = state.pool.get()?;
        let balances = wallet::db::balance::get_for_all_nodes_by_unit(&db_conn, unit)?;

        let mut used_node = vec![];
        let mut rem_amount = amount;
        for balance in balances {
            match rem_amount.cmp(&balance.amount) {
                Ordering::Less | Ordering::Equal => {
                    used_node.push((balance.id, balance.url, rem_amount));
                    rem_amount = Amount::ZERO;
                    break;
                }
                Ordering::Greater => {
                    rem_amount -= balance.amount;
                    used_node.push((balance.id, balance.url, balance.amount));
                }
            }
        }

        if rem_amount != Amount::ZERO {
            return Err(CreateWadsError::NotEnoughFunds(amount, rem_amount));
        }

        used_node
    };

    let mut wads = Vec::with_capacity(amount_to_use_per_node.len());
    let mut balance_decrease_events = Vec::with_capacity(amount_to_use_per_node.len());
    for (node_id, node_url, amount_to_use) in amount_to_use_per_node {
        let mut node_client = wallet::connect_to_node(&node_url).await?;

        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            state.pool.clone(),
            &mut node_client,
            node_id,
            amount_to_use,
            unit.as_str(),
        )
        .await?
        .ok_or(CreateWadsError::NotEnoughFundsInNode(node_id))?;

        let db_conn = state.pool.get()?;
        let proofs = wallet::load_tokens_from_db(&db_conn, &proofs_ids)?;
        let memo = None;
        wallet::db::wad::register_wad(
            &db_conn,
            wallet::db::wad::WadType::OUT,
            &node_url,
            &memo,
            &proofs_ids,
        )?;
        let wad = wallet::wad::create_from_proofs(node_url, unit, memo, proofs);
        wads.push(wad);
        balance_decrease_events.push(BalanceChange {
            node_id,
            unit: unit.as_str().to_string(),
            amount: amount_to_use.into(),
        });
    }
    for event in balance_decrease_events {
        app.emit("balance-decrease", event)?;
    }

    Ok(CompactWads(wads).to_string())
}
