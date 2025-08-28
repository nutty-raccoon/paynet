use std::str::FromStr;

use nuts::traits::Unit as UnitT;
use starknet_types::Asset;
use tauri::State;
use wallet::{db::balance::Balance, types::NodeUrl};

use crate::AppState;

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
pub async fn add_node(
    state: State<'_, AppState>,
    node_url: String,
) -> Result<(u32, Vec<Balance>), AddNodeError> {
    let node_url = NodeUrl::from_str(&node_url)?;
    let mut client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert()).await?;
    let id = wallet::node::register(state.pool.clone(), &mut client, &node_url).await?;

    let wallet = wallet::db::wallet::get(&*state.pool.get()?)?.unwrap();

    if wallet.is_restored {
        wallet::node::restore(crate::SEED_PHRASE_MANAGER, state.pool.clone(), id, client).await?;
    }

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

    Ok((id, balances))
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshNodeKeysetsError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    NodeConnect(#[from] wallet::ConnectToNodeError),
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
pub async fn refresh_node_keysets(
    state: State<'_, AppState>,
    node_id: u32,
) -> Result<(), RefreshNodeKeysetsError> {
    let node_url = {
        let db_conn = state.pool.get()?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(RefreshNodeKeysetsError::NodeId(node_id))?
    };
    let mut node_client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert()).await?;
    wallet::node::refresh_keysets(state.pool.clone(), &mut node_client, node_id)
        .await
        .map_err(|e| RefreshNodeKeysetsError::Wallet(node_id, e))?;

    Ok(())
}
