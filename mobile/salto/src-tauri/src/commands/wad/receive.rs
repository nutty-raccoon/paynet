use std::{collections::HashSet, str::FromStr};

use nuts::traits::Unit as UnitT;
use starknet_types::{Asset, Unit};
use tauri::{AppHandle, State};
use wallet::types::compact_wad::{self, CompactWad, CompactWads};

use crate::{
    AppState,
    errors::CommonError,
    front_events::{BalanceChange, emit_balance_increase_event},
};

#[derive(Debug, thiserror::Error)]
pub enum ReceiveWadsError {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("invalid string for compacted wad: {0}")]
    WadString(#[from] compact_wad::Error),
    #[error("failed to register node: {0}")]
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
    let wads: CompactWads = wads.parse()?;
    let mut new_assets: HashSet<Asset> = HashSet::new();

    for wad in wads.0 {
        let CompactWad {
            node_url,
            unit,
            memo,
            proofs,
        } = wad;
        let mut node_client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert())
            .await
            .map_err(CommonError::CreateNodeClient)?;
        let node_id =
            wallet::node::register(state.pool.clone(), &mut node_client, &node_url).await?;

        let amount_received = wallet::receive_wad(
            crate::SEED_PHRASE_MANAGER,
            state.pool.clone(),
            &mut node_client,
            node_id,
            &node_url,
            unit.as_str(),
            proofs,
            &memo,
        )
        .await
        .map_err(CommonError::Wallet)?;

        if let Ok(unit) = Unit::from_str(&unit) {
            new_assets.insert(unit.matching_asset());
        }

        emit_balance_increase_event(
            &app,
            BalanceChange {
                node_id,
                unit,
                amount: amount_received.into(),
            },
        )
        .map_err(CommonError::EmitTauriEvent)?;
    }

    state
        .get_prices_config
        .write()
        .await
        .assets
        .extend(new_assets);

    Ok(())
}
