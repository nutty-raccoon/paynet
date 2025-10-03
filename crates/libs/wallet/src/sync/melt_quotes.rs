use std::time::{SystemTime, UNIX_EPOCH};

use node_client::UnspecifiedEnum;
use nuts::nut05::MeltQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::Code;
use tracing::{Level, debug, event};

use crate::{
    db::{self, melt_quote::PendingMeltQuote},
    melt::format_melt_transfers_id_into_term_message,
};

#[derive(Debug, thiserror::Error)]
pub enum SyncMeltQuotesError {
    #[error("failed to sync melt quote with id {0}: {1}")]
    SyncOne(String, SyncMeltQuoteError),
}

#[derive(Debug, Default, Clone)]
pub struct MeltQuotesStateUpdate {
    pub deleted: Vec<String>,
    pub unchanged: Vec<PendingMeltQuote>,
    pub changed: Vec<PendingMeltQuote>,
}

pub async fn melt_quotes(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl cashu_client::CashuClient,
    pending_melt_quotes: Vec<PendingMeltQuote>,
) -> Result<MeltQuotesStateUpdate, SyncMeltQuotesError> {
    let mut states_updates = MeltQuotesStateUpdate::default();
    for pending_melt_quote in pending_melt_quotes {
        let (new_state, tx_ids) = {
            match melt_quote(
                pool.clone(),
                node_client,
                pending_melt_quote.method.clone(),
                pending_melt_quote.id.clone(),
            )
            .await
            {
                Ok(opt) => match opt {
                    Some(new_state) => new_state,
                    None => {
                        states_updates.deleted.push(pending_melt_quote.id);
                        continue;
                    }
                },
                Err(e) => return Err(SyncMeltQuotesError::SyncOne(pending_melt_quote.id, e)),
            }
        };

        if MeltQuoteState::Paid == new_state {
            event!(name: "melt-quote-paid", Level::INFO,
                quote_id=pending_melt_quote.id,
                tx_ids=format_melt_transfers_id_into_term_message(tx_ids),
                "Melt quote paid"
            );
        }

        if new_state == pending_melt_quote.state {
            states_updates.unchanged.push(pending_melt_quote);
        } else {
            let mut pending_melt_quote = pending_melt_quote;
            pending_melt_quote.state = new_state;
            states_updates.changed.push(pending_melt_quote);
        }
    }

    Ok(states_updates)
}

#[derive(Debug, thiserror::Error)]
pub enum SyncMeltQuoteError {
    #[error("failed to get connection from the pool")]
    Pool(#[from] r2d2::Error),
    #[error("invalid mint quote state: {0}")]
    InvalidState(String),
    #[error("invalid mint quote state: {0}")]
    BadEnum(#[from] UnspecifiedEnum),
    #[error("failed to delete quote: {0}")]
    Delete(rusqlite::Error),
    #[error("failed to set quote state: {0}")]
    SetState(rusqlite::Error),
    #[error("failed register transfers ids: {0}")]
    RegisterTransferIds(rusqlite::Error),
    #[error("failed to interact with the node: {0}")]
    Client(#[from] cashu_client::Error),
    #[error("failed to start database transaction: {0}")]
    StartDbTransaction(#[source] rusqlite::Error),
    #[error("failed to commit database transaction: {0}")]
    CommitDbTransaction(#[source] rusqlite::Error),
    #[error("failed to serialize transfer ids: {0}")]
    SerializeTransferIds(#[from] serde_json::Error),
}

/// Sync the state of this quote from the node.
///
/// 1. query the node for the state
/// 2. delete expired quote
/// 3. update state in database
///
/// Returns node if the melt quote has been deleted,
/// otherwise returns its current state.
pub async fn melt_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl cashu_client::CashuClient,
    method: String,
    quote_id: String,
) -> Result<Option<(MeltQuoteState, Vec<String>)>, SyncMeltQuoteError> {
    let response = node_client.melt_quote_state(method, quote_id.clone()).await;

    let mut db_conn = pool.get()?;
    match response {
        Ok(response) => {
            let state =
                MeltQuoteState::try_from(node_client::MeltQuoteState::from(response.state))?;

            let tx = db_conn
                .transaction()
                .map_err(SyncMeltQuoteError::StartDbTransaction)?;
            match state {
                MeltQuoteState::Unpaid => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now >= response.expiry {
                        db::melt_quote::delete(&tx, &quote_id)
                            .map_err(SyncMeltQuoteError::Delete)?;
                        return Ok(None);
                    }
                }
                MeltQuoteState::Pending => {}
                MeltQuoteState::Paid => {
                    if response.transfer_ids.is_some() {
                        let transfer_ids_to_store = serde_json::to_string(&response.transfer_ids)?;
                        db::melt_quote::register_transfer_ids(
                            &tx,
                            &quote_id,
                            &transfer_ids_to_store,
                        )
                        .map_err(SyncMeltQuoteError::RegisterTransferIds)?;
                    }
                }
            }

            db::melt_quote::set_state(&tx, &quote_id, response.state)
                .map_err(SyncMeltQuoteError::SetState)?;
            tx.commit()
                .map_err(SyncMeltQuoteError::CommitDbTransaction)?;

            Ok(Some((state, response.transfer_ids.unwrap_or_default())))
        }
        Err(cashu_client::Error::Grpc(s)) if s.code() == Code::NotFound => {
            db::mint_quote::delete(&db_conn, &quote_id).map_err(SyncMeltQuoteError::Delete)?;
            Ok(None)
        }
        Err(e) => {
            debug!("got error from node: {e}");
            Err(e)?
        }
    }
}
