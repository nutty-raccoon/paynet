use nuts::nut04::MintQuoteState;
use tauri::{AppHandle, State};
use wallet::db::balance::GetForAllNodesData;
use wallet::{ConnectToNodeError, connect_to_node};

use crate::AppState;

use super::deposit::inner_redeem_quote;

#[derive(Debug, thiserror::Error)]
pub enum GetNodesBalanceError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
}

impl serde::Serialize for GetNodesBalanceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn get_nodes_balance(
    state: State<'_, AppState>,
) -> Result<Vec<GetForAllNodesData>, GetNodesBalanceError> {
    let db_conn = state.pool.get()?;
    let nodes_balances = wallet::db::balance::get_for_all_nodes(&db_conn)?;
    Ok(nodes_balances)
}

#[derive(Debug, thiserror::Error)]
pub enum GetPendingMintQuoteError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error("failed to connect to node: {0}")]
    NodeConnect(#[from] ConnectToNodeError),
    #[error(transparent)]
    Wallet(#[from] ::wallet::errors::Error),
}

#[tauri::command]
pub async fn get_pending_mint_quotes(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), GetPendingMintQuoteError> {
    let db_conn = state.pool.get()?;
    let pending_mint_quotes = wallet::db::mint_quote::get_pendings(&db_conn)?;

    for (node_id, pending_mint_quotes) in pending_mint_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?.unwrap();
        let mut node_client = connect_to_node(&node_url, state.opt_root_ca_cert()).await?;
        for pending_mint_quote in pending_mint_quotes {
            let new_state = {
                match wallet::sync::mint_quote(
                    state.pool.clone(),
                    &mut node_client,
                    pending_mint_quote.method.clone(),
                    pending_mint_quote.id.clone(),
                )
                .await?
                {
                    Some(new_state) => new_state,
                    None => {
                        // TODO: emit event
                        continue;
                    }
                }
            };

            if new_state == MintQuoteState::Paid {
                match inner_redeem_quote(
                    &app,
                    state.clone(),
                    &mut node_client,
                    node_id,
                    &pending_mint_quote.id,
                    pending_mint_quote.unit,
                    pending_mint_quote.amount,
                )
                .await
                {
                    Ok(_) => todo!(),
                    Err(e) => log::error!(
                        "failed to redeem the quote with id {}: {e}",
                        pending_mint_quote.id
                    ),
                }
            }
        }
    }

    Ok(())
}
