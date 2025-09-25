use std::{collections::HashSet, str::FromStr};

use nuts::traits::Unit as UnitT;
use starknet_types::{Asset, Unit};
use tauri::{AppHandle, State};
use tracing::{Level, event};
use wallet::types::compact_wad::{self, CompactWad, CompactWads};

use crate::{AppState, errors::CommonError, front_events::emit_trigger_balance_poll};

#[derive(Debug, thiserror::Error)]
pub enum ReceiveWadsError {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("invalid string for compacted wad: {0}")]
    WadString(#[from] compact_wad::Error),
    #[error("failed to register node: {0}")]
    RegisterNode(#[from] wallet::node::RegisterNodeError),
    #[error("failed to create node client: {0}")]
    CreateNodeClient(wallet::ConnectToNodeError),
    #[error(transparent)]
    ReceiveWad(#[from] wallet::ReceiveWadError),
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
#[tracing::instrument(skip(app, state))]
pub async fn receive_wads(
    app: AppHandle,
    state: State<'_, AppState>,
    wads: String,
) -> Result<(), ReceiveWadsError> {
    let wads: CompactWads = wads.parse()?;
    let mut new_assets: HashSet<Asset> = HashSet::new();

    event!(name: "wads_parsed", Level::INFO,
        num_wads = wads.0.len(),
        "Wad parsed"
    );

    for wad in wads.0 {
        let CompactWad {
            node_url,
            unit,
            memo,
            proofs,
        } = wad;

        event!(name: "processing_wad", Level::INFO,
            node_url = %node_url,
            unit = %unit,
            num_proofs = proofs.len(),
            "Processing wad"
        );

        // Try to find existing node_id by URL first
        let existing_node_id = {
            let db_conn = state.pool().get().map_err(CommonError::DbPool)?;
            wallet::db::node::get_id_by_url(&db_conn, &node_url).map_err(CommonError::Db)?
        };

        let (mut node_client, node_id) = if let Some(existing_node_id) = existing_node_id {
            // Use cached connection if node exists
            event!(name: "known_node", Level::INFO,
                node_id = existing_node_id,
                node_url = %node_url,
                "Using known node"
            );
            let client = state
                .get_node_client_connection(existing_node_id)
                .await
                .map_err(CommonError::CachedConnection)?;
            (client, existing_node_id)
        } else {
            // Create direct connection and register new node
            event!(name: "registering_new_node", Level::INFO,
                node_url = %node_url,
                "Registering new node"
            );
            let mut client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert())
                .await
                .map_err(ReceiveWadsError::CreateNodeClient)?;
            let node_id =
                wallet::node::register(state.pool().clone(), &mut client, &node_url).await?;
            event!(name: "new_node_registered", Level::INFO,
                node_id = node_id,
                node_url = %node_url,
                "New node registered"
            );
            (client, node_id)
        };

        event!(name: "receiving_wad", Level::INFO,
            node_id = node_id,
            unit = %unit,
            num_proofs = proofs.len(),
            "Reciving wad"
        );

        let amount_received = wallet::receive_wad(
            crate::SEED_PHRASE_MANAGER,
            state.pool().clone(),
            &mut node_client,
            node_id,
            &node_url,
            unit.as_str(),
            proofs,
            &memo,
        )
        .await?;

        event!(name: "wad_received", Level::INFO,
            node_id = node_id,
            unit = %unit,
            amount_received = %amount_received,
            "Wad received"
        );

        if let Ok(unit) = Unit::from_str(&unit) {
            new_assets.insert(unit.matching_asset());
        }
    }

    state
        .get_prices_config()
        .write()
        .await
        .assets
        .extend(new_assets);

    // Trigger immediate balance polling after receiving all wads
    let _ = emit_trigger_balance_poll(&app);

    Ok(())
}
