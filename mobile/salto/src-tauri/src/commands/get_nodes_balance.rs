use nuts::nut04::MintQuoteState;
use nuts::nut05::MeltQuoteState;
use tauri::{AppHandle, Manager, State, async_runtime};
use tracing::{error, instrument};
use wallet::db::balance::GetForAllNodesData;
use wallet::db::melt_quote::PendingMeltQuote;
use wallet::db::mint_quote::PendingMintQuote;

use crate::AppState;
use crate::errors::CommonError;
use crate::front_events::{PendingQuoteData, emit_trigger_balance_poll};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MintOrMeltQuote {
    Mint(MintPendingQuotes),
    Melt(MeltPendingQuotes),
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintPendingQuotes {
    pub node_id: u32,
    pub unpaid: Vec<PendingQuoteData>,
    pub paid: Vec<PendingQuoteData>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeltPendingQuotes {
    pub node_id: u32,
    pub unpaid: Vec<PendingQuoteData>,
    pub pending: Vec<PendingQuoteData>,
}

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

#[instrument(skip(state))]
#[tauri::command]
pub async fn get_nodes_balance(
    state: State<'_, AppState>,
) -> Result<Vec<GetForAllNodesData>, GetNodesBalanceError> {
    let db_conn = state.pool.get()?;
    let nodes_balances = wallet::db::balance::get_for_all_nodes(&db_conn)?;
    Ok(nodes_balances)
}

#[derive(Debug, thiserror::Error)]
pub enum GetPendingQuoteError {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("failed to get the pendings quote form the db: {0}")]
    ReadPendingsFromDb(rusqlite::Error),
    #[error("failed to join the handles: {0}")]
    JoinAll(tauri::Error),
}

impl serde::Serialize for GetPendingQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[instrument(skip(app, state))]
#[tauri::command]
pub async fn get_pending_quotes(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<MintOrMeltQuote>, GetPendingQuoteError> {
    let (pending_mint_quotes, pending_melt_quotes) = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        let mint_quotes = wallet::db::mint_quote::get_pendings(&db_conn)
            .map_err(GetPendingQuoteError::ReadPendingsFromDb)?;
        let melt_quotes = wallet::db::melt_quote::get_pendings(&db_conn)
            .map_err(GetPendingQuoteError::ReadPendingsFromDb)?;
        (mint_quotes, melt_quotes)
    };

    let mut handles = Vec::with_capacity(pending_mint_quotes.len() + pending_melt_quotes.len());
    for (node_id, pending_mint_quotes) in pending_mint_quotes {
        let app_handle = app.clone();

        handles.push(async_runtime::spawn(sync_node_mint_quotes(
            app_handle,
            node_id,
            pending_mint_quotes,
        )));
    }
    for (node_id, pending_melt_quotes) in pending_melt_quotes {
        let app_handle = app.clone();

        handles.push(async_runtime::spawn(sync_node_melt_quotes(
            app_handle,
            node_id,
            pending_melt_quotes,
        )));
    }

    let mut quotes = Vec::new();
    for results in futures::future::join_all(handles).await {
        match results.map_err(GetPendingQuoteError::JoinAll)? {
            Ok(quote) => quotes.push(quote),
            Err(e) => error!("failed to sync node: {e}"),
        }
    }

    Ok(quotes)
}

#[derive(Debug, thiserror::Error)]
pub enum SyncNodeError {
    #[error(transparent)]
    Common(#[from] CommonError),
    #[error("failed to sync the mint quotes of node {0}: {1}")]
    SyncMint(u32, wallet::sync::SyncMintQuotesError),
    #[error("failed to sync the melt quotes of node {0}: {1}")]
    SyncMelt(u32, wallet::sync::SyncMeltQuotesError),
}

#[instrument(skip(app))]
async fn sync_node_mint_quotes(
    app: AppHandle,
    node_id: u32,
    pending_mint_quotes: Vec<PendingMintQuote>,
) -> Result<MintOrMeltQuote, SyncNodeError> {
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
        false,
    )
    .await
    .map_err(|e| SyncNodeError::SyncMint(node_id, e))?;

    let mut unpaid = Vec::new();
    let mut paid = Vec::new();
    let mut any_issued = false;
    for mint_quote in state_updates.unchanged.into_iter() {
        match mint_quote.state {
            MintQuoteState::Unpaid => unpaid.push(PendingQuoteData {
                id: mint_quote.id,
                unit: mint_quote.unit,
                amount: mint_quote.amount.into(),
            }),
            MintQuoteState::Paid => paid.push(PendingQuoteData {
                id: mint_quote.id,
                unit: mint_quote.unit,
                amount: mint_quote.amount.into(),
            }),
            MintQuoteState::Issued => {
                unreachable!("issued quote wouldn't have been selected as pending")
            }
        }
    }
    for mint_quote in state_updates.changed.into_iter() {
        match mint_quote.state {
            MintQuoteState::Unpaid => {
                unreachable!("the state cannot have changed and now be unpaid")
            }
            MintQuoteState::Paid => paid.push(PendingQuoteData {
                id: mint_quote.id,
                unit: mint_quote.unit,
                amount: mint_quote.amount.into(),
            }),
            MintQuoteState::Issued => {
                any_issued = true;
            }
        }
    }

    if any_issued {
        let _ = emit_trigger_balance_poll(&app);
    }

    Ok(MintOrMeltQuote::Mint(MintPendingQuotes {
        unpaid,
        paid,
        node_id,
    }))
}

#[instrument(skip(app))]
async fn sync_node_melt_quotes(
    app: AppHandle,
    node_id: u32,
    pending_melt_quotes: Vec<PendingMeltQuote>,
) -> Result<MintOrMeltQuote, SyncNodeError> {
    let state = app.state::<AppState>();
    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    let state_updates =
        wallet::sync::melt_quotes(state.pool.clone(), &mut node_client, pending_melt_quotes)
            .await
            .map_err(|e| SyncNodeError::SyncMelt(node_id, e))?;

    let mut unpaid = Vec::new();
    let mut pending = Vec::new();
    let mut any_deposited = false;
    for melt_quote in state_updates.unchanged.into_iter() {
        match melt_quote.state {
            MeltQuoteState::Unpaid => unpaid.push(PendingQuoteData {
                id: melt_quote.id,
                unit: melt_quote.unit,
                amount: melt_quote.amount.into(),
            }),
            MeltQuoteState::Pending => pending.push(PendingQuoteData {
                id: melt_quote.id,
                unit: melt_quote.unit,
                amount: melt_quote.amount.into(),
            }),
            MeltQuoteState::Paid => {
                unreachable!("Paid quote wouldn't have been selected as pending")
            }
        }
    }
    for melt_quote in state_updates.changed {
        match melt_quote.state {
            MeltQuoteState::Unpaid => {
                unreachable!("the state cannot have changed and now be unpaid")
            }
            MeltQuoteState::Pending => any_deposited = true,
            MeltQuoteState::Paid => {}
        }
    }

    if any_deposited {
        let _ = emit_trigger_balance_poll(&app);
    }

    Ok(MintOrMeltQuote::Melt(MeltPendingQuotes {
        unpaid,
        pending,
        node_id,
    }))
}
