use std::{cmp::Ordering, str::FromStr};

use nuts::Amount;
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, Unit};
use tauri::{AppHandle, Emitter, State};
use wallet::types::compact_wad::{self, CompactWad};

use crate::{
    parse_asset_amount::{parse_asset_amount, ParseAmountStringError},
    AppState,
};

use super::BalanceChange;

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
        let wad = wallet::create_wad_from_proofs(node_url, unit, None, proofs);
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

    let wads_string = serde_json::to_string(&wads)?;
    Ok(wads_string)
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveWadsError {
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
    #[error("invalid string for compacted wad")]
    WadString(#[from] compact_wad::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("this is a json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl serde::Serialize for ReceiveWadsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn receive_wads(
    app: AppHandle,
    state: State<'_, AppState>,
    wads: String,
) -> Result<(), ReceiveWadsError> {
    let deserialized_wads: Vec<CompactWad<Unit>> = serde_json::from_str(&wads)?;

    for wad in deserialized_wads {
        let (mut node_client, node_id) =
            wallet::register_node(state.pool.clone(), &wad.node_url).await?;

        let amount_received = wallet::receive_wad(
            state.pool.clone(),
            &mut node_client,
            node_id,
            wad.unit.as_str(),
            wad.proofs,
        )
        .await?;

        app.emit(
            "balance-increase",
            BalanceChange {
                node_id,
                unit: wad.unit.as_str().to_string(),
                amount: amount_received.into(),
            },
        )?;
    }

    Ok(())
}
