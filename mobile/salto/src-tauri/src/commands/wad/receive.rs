use std::collections::HashSet;

use starknet_types::{AssetFromStrError, AssetToUnitConversionError, Unit};
use tauri::{AppHandle, Emitter, State};
use wallet::types::compact_wad::{self, CompactWad, CompactWads};

use crate::{AppState, commands::BalanceChange, parse_asset_amount::ParseAmountStringError};

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
    #[error(transparent)]
    RegisterNode(#[from] wallet::node::RegisterNodeError),
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
    let wads: CompactWads<Unit> = wads.parse()?;
    let mut new_assets: HashSet<String> = HashSet::new();

    for wad in wads.0 {
        let CompactWad {
            node_url,
            unit,
            memo,
            proofs,
        } = wad;
        let (mut node_client, node_id) =
            wallet::node::register(state.pool.clone(), &node_url).await?;

        let amount_received = wallet::receive_wad(
            state.pool.clone(),
            &mut node_client,
            node_id,
            &node_url,
            unit.as_str(),
            proofs,
            &memo,
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
        new_assets.insert(wad.unit.as_str().to_string());
    }

    state
        .get_prices_config
        .write()
        .await
        .assets
        .extend(new_assets);

    Ok(())
}
