use nuts::nut04::MintQuoteState;
use tauri::{AppHandle, Manager, State, async_runtime};
use wallet::db::balance::GetForAllNodesData;
use wallet::db::mint_quote::PendingMintQuote;

use crate::AppState;
use crate::errors::CommonError;
use crate::front_events::{
    NodePendingMintQuotesStateUpdatesEvent, PendingMintQuoteData,
    emit_node_pending_mint_quotes_updates_event,
};

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
    Common(#[from] CommonError),
    #[error("failed to get the pendings mint quote form the db: {0}")]
    ReadPendingsFromDb(rusqlite::Error),
}

impl serde::Serialize for GetPendingMintQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn get_pending_mint_quotes(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), GetPendingMintQuoteError> {
    let pending_mint_quotes = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::mint_quote::get_pendings(&db_conn)
            .map_err(GetPendingMintQuoteError::ReadPendingsFromDb)?
    };

    for (node_id, pending_mint_quotes) in pending_mint_quotes {
        let app_handle = app.clone();

        async_runtime::spawn(sync_node(app_handle, node_id, pending_mint_quotes));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SyncNodeError {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("failed to sync the mint quotes of node {0}: {1}")]
    Sync(u32, wallet::sync::SyncMintQuotesError),
}

async fn sync_node(
    app: AppHandle,
    node_id: u32,
    pending_mint_quotes: Vec<PendingMintQuote>,
) -> Result<(), SyncNodeError> {
    let state = app.state::<AppState>();
    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;
    let state_updates = wallet::sync::mint_quotes(
        crate::SEED_PHRASE_MANAGER,
        state.pool.clone(),
        &mut node_client,
        node_id,
        pending_mint_quotes,
    )
    .await
    .map_err(|e| SyncNodeError::Sync(node_id, e))?;

    let mut unpaid = Vec::new();
    let mut paid = Vec::new();
    for mint_quote in state_updates
        .unchanged
        .into_iter()
        .chain(state_updates.changed)
    {
        if mint_quote.state == MintQuoteState::Unpaid {
            unpaid.push(PendingMintQuoteData {
                id: mint_quote.id,
                unit: mint_quote.unit,
                amount: mint_quote.amount.into(),
            });
        } else if mint_quote.state == MintQuoteState::Paid {
            paid.push(PendingMintQuoteData {
                id: mint_quote.id,
                unit: mint_quote.unit,
                amount: mint_quote.amount.into(),
            });
        }
    }

    emit_node_pending_mint_quotes_updates_event(
        &app,
        NodePendingMintQuotesStateUpdatesEvent {
            node_id,
            unpaid,
            paid,
        },
    )
    .map_err(CommonError::EmitTauriEvent)?;

    Ok(())
}
