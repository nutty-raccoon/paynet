use std::str::FromStr;

use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, Unit};
use tauri::State;
use wallet::types::compact_wad::{self, CompactWad};

use crate::{
    parse_asset_amount::{parse_asset_amount, ParseAmountStringError},
    AppState,
};

use super::BalanceIncrease;

#[derive(Debug, thiserror::Error)]
pub enum CreateWadError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error("unknown node_id: {0}")]
    NodeId(u32),
    #[error(transparent)]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error(transparent)]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error("not enough funds")]
    NotEnoughFunds,
}

impl serde::Serialize for CreateWadError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn create_wad(
    state: State<'_, AppState>,
    node_id: u32,
    amount: String,
    asset: String,
) -> Result<CompactWad<Unit>, CreateWadError> {
    let asset = Asset::from_str(&asset)?;
    let unit = asset.find_best_unit();
    let amount = parse_asset_amount(&amount, asset, unit)?;

    let node_url = {
        let db_conn = state.pool.get()?;
        wallet::db::get_node_url(&db_conn, node_id)?.ok_or(CreateWadError::NodeId(node_id))?
    };
    let mut node_client = wallet::connect_to_node(&node_url).await?;

    let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
        state.pool.clone(),
        &mut node_client,
        node_id,
        amount,
        unit.as_str(),
    )
    .await?
    .ok_or(CreateWadError::NotEnoughFunds)?;

    let db_conn = state.pool.get()?;
    let proofs = wallet::load_tokens_from_db(&db_conn, proofs_ids)?;
    let wad = wallet::create_wad_from_proofs(node_url, unit, None, proofs);

    Ok(wad)
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveWadError {
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
}

impl serde::Serialize for ReceiveWadError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn receive_wad(
    state: State<'_, AppState>,
    wad_string: String,
) -> Result<BalanceIncrease, ReceiveWadError> {
    let wad: CompactWad<Unit> = wad_string.parse()?;

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

    Ok(BalanceIncrease {
        node_id,
        unit: wad.unit.as_str().to_string(),
        amount: amount_received.into(),
    })
}
