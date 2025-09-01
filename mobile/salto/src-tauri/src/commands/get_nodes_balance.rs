use nuts::nut04::MintQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::{AppHandle, Emitter, State, async_runtime};
use tonic::transport::Certificate;
use wallet::db::balance::GetForAllNodesData;
use wallet::db::mint_quote::PendingMintQuote;
use wallet::{ConnectToNodeError, connect_to_node};

use crate::AppState;

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
    #[error("failed to get connection from pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("failed to get the pendings mint quote form the db: {0}")]
    ReadPendingsFromDb(rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] ::wallet::errors::Error),
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
        let db_conn = state.pool.get()?;
        wallet::db::mint_quote::get_pendings(&db_conn)
            .map_err(GetPendingMintQuoteError::ReadPendingsFromDb)?
    };

    let opt_root_ca_cert = state.opt_root_ca_cert();
    for (node_id, pending_mint_quotes) in pending_mint_quotes {
        let app_handle = app.clone();
        let pool = state.pool.clone();
        let opt_root_ca_cert = opt_root_ca_cert.clone();
        async_runtime::spawn(sync_node(app_handle, pool, opt_root_ca_cert, node_id, pending_mint_quotes));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum SyncNodeError {
    #[error("failed to get connection from pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("failed to get the node url from the db: {0}")]
    GetNodeUrl(rusqlite::Error),
    #[error("failed to connect to node: {0}")]
    NodeConnect(#[from] ConnectToNodeError),
    #[error("failed to sync the mint quotes of node {0}: {1}")]
    Sync(u32, wallet::sync::SyncMintQuotesError),
    #[error("failed to emmit event: {0}")]
    Tauri(#[from] tauri::Error),
}

async fn sync_node(
    app: AppHandle,
    pool: Pool<SqliteConnectionManager>,
    opt_root_ca_cert: Option<Certificate>,
    node_id: u32,
    pending_mint_quotes: Vec<PendingMintQuote>,
) -> Result<(), SyncNodeError> {
    let node_url = {
        let db_conn = pool.get()?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)
            .map_err(SyncNodeError::GetNodeUrl)?
            .unwrap()
    };
    let mut node_client = connect_to_node(&node_url, opt_root_ca_cert).await?;
    let state_updates = wallet::sync::mint_quotes(
        crate::SEED_PHRASE_MANAGER,
        pool.clone(),
        &mut node_client,
        node_id,
        pending_mint_quotes,
    )
    .await
    .map_err(|e| SyncNodeError::Sync(node_id, e))?;

    let mut unpaid = Vec::new();
    let mut paid = Vec::new();
    for (id, state) in state_updates
        .unchanged
        .into_iter()
        .chain(state_updates.changed)
    {
        if state == MintQuoteState::Unpaid {
            unpaid.push(id);
        } else if state == MintQuoteState::Paid {
            paid.push(id);
        }
    }

    app.emit(
        "pending-mint-quote-updated",
        NodePendingMintQuotesStateUpdatesEvent {
            node_id,
            unpaid,
            paid,
        },
    )?;

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodePendingMintQuotesStateUpdatesEvent {
    node_id: u32,
    unpaid: Vec<String>,
    paid: Vec<String>,
}
