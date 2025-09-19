use std::str::FromStr;

use nuts::traits::Unit as UnitT;
use starknet_types::Asset;
use tauri::{AppHandle, State};
use tracing::{Level, event, warn};
use wallet::types::NodeUrl;

use crate::{AppState, front_events::emit_trigger_balance_poll};

#[derive(Debug, thiserror::Error)]
pub enum AddNodeError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error("invalid node url: {0}")]
    InvalidNodeUrl(#[from] wallet::types::NodeUrlError),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error), // TODO: create more granular errors in wallet
    #[error("failed to register node: {0}")]
    RegisterNode(#[from] wallet::node::RegisterNodeError),
    #[error("failed to restore node: {0}")]
    RestoreNode(#[from] wallet::node::RestoreNodeError),
    #[error("invalid private key stored in db: {0}")]
    Bip32(#[from] bitcoin::bip32::Error),
    #[error("failed to connect to node: {0}")]
    ConnectToNode(#[from] wallet::ConnectToNodeError),
    #[error("failed parse db unit: {0}")]
    Unit(#[from] starknet_types::UnitFromStrError),
}

impl serde::Serialize for AddNodeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub async fn add_node(
    app: AppHandle,
    state: State<'_, AppState>,
    node_url: String,
) -> Result<(), AddNodeError> {
    let node_url = NodeUrl::from_str(&node_url)?;

    event!(name: "connecting_to_node", Level::INFO,
        node_url = %node_url,
        "Connecting to node"
    );

    let mut client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert()).await?;

    event!(name: "registering_node", Level::INFO,
        node_url = %node_url,
        "Registering node"
    );

    let id = wallet::node::register(state.pool.clone(), &mut client, &node_url).await?;

    event!(name: "node_registered_successfully", Level::INFO,
        node_id = id,
        node_url = %node_url,
        "Node registered"
    );

    event!(name: "restoring_node_from_wallet", Level::INFO, node_id = id, "Restoring node");
    wallet::node::restore(crate::SEED_PHRASE_MANAGER, state.pool.clone(), id, client).await?;
    event!(name: "node_restore_completed", Level::INFO, node_id = id, "Node restored");

    let _ = emit_trigger_balance_poll(&app);

    let balances = wallet::db::balance::get_for_node(&*state.pool.get()?, id)?;
    let new_assets = balances
        .clone()
        .into_iter()
        .map(|b| -> Result<Asset, _> {
            starknet_types::Unit::from_str(&b.unit).map(|u| u.matching_asset())
        })
        .collect::<Result<Vec<_>, _>>()?;
    state
        .get_prices_config
        .write()
        .await
        .assets
        .extend(new_assets);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ForgetNodeError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(
        "There are still funds deposited to this node. Withdraw them first, or call again with 'force = true'"
    )]
    HasFunds,
}

impl serde::Serialize for ForgetNodeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub fn forget_node(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: u32,
    force: bool,
) -> Result<(), ForgetNodeError> {
    let db_conn = state.pool.get()?;

    if !force {
        let balances = wallet::db::balance::get_for_node(&db_conn, node_id)?;
        let has_no_funds = balances.is_empty();
        if !has_no_funds {
            return Err(ForgetNodeError::HasFunds);
        }
    }

    let n_row_deleted = wallet::db::node::delete_by_id(&db_conn, node_id)?;
    if n_row_deleted != 1 {
        warn!("unexpected number of node deleted: {}", n_row_deleted);
    } else {
        event!(name: "node_forgotten", Level::INFO, node_id);
    }

    let _ = emit_trigger_balance_poll(&app);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshNodeKeysetsError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error("unknown node_id: {0}")]
    NodeId(u32),
    #[error("fail to refresh the node {0} keyset: {1}")]
    Wallet(u32, wallet::node::RefreshNodeKeysetError),
}

impl serde::Serialize for RefreshNodeKeysetsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn refresh_node_keysets(
    state: State<'_, AppState>,
    node_id: u32,
) -> Result<(), RefreshNodeKeysetsError> {
    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(|_| RefreshNodeKeysetsError::NodeId(node_id))?;

    event!(name: "refresh_node_keysets", Level::INFO, node_id, "Refreshing keyset");
    wallet::node::refresh_keysets(state.pool.clone(), &mut node_client, node_id)
        .await
        .map_err(|e| RefreshNodeKeysetsError::Wallet(node_id, e))?;

    event!(name: "node_keysets_refreshed", Level::INFO,
        node_id = node_id,
        "Keyset refreshed"
    );

    Ok(())
}
