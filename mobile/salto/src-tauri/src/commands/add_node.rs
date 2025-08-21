use std::str::FromStr;

use bitcoin::bip32::Xpriv;
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
    #[error("invalid private key stored in db: {0}")]
    Bip32(#[from] bitcoin::bip32::Error),
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
        let xpriv = Xpriv::from_str(&wallet.private_key)?;
        wallet::node::restore(state.pool.clone(), id, client, xpriv).await?;
    }

    let balances = wallet::db::balance::get_for_node(&*state.pool.get()?, id)?;
    let new_assets = balances
        .clone()
        .into_iter()
        .map(|b| {
            if b.unit.to_lowercase() == "millistrk" {
                return "strk".to_string();
            }
            b.unit.to_lowercase()
        })
        .collect::<Vec<_>>();
    state
        .get_prices_config
        .write()
        .await
        .assets
        .extend(new_assets);

    Ok((id, balances))
}
