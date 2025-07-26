use std::str::FromStr;

use tauri::State;
use wallet::{db::balance::Balance, types::NodeUrl};

use crate::AppState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
}

impl serde::Serialize for Error {
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
) -> Result<(u32, Vec<Balance>), Error> {
    let node_url = NodeUrl::from_str(&node_url)?;
    let (client, id) = wallet::node::register(state.pool.clone(), &node_url).await?;

    let wallet = wallet::db::wallet::get(&*state.pool.get()?)?.unwrap();

    if wallet.is_restored {
        wallet::node::restore(state.pool.clone(), id, client, wallet.private_key).await?;
    }

    let balances = wallet::db::balance::get_for_node(&*state.pool.get()?, id)?;

    Ok((id, balances))
}
