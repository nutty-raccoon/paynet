use std::time::{SystemTime, UNIX_EPOCH};

use node_client::{NodeClient, QuoteStateRequest, UnspecifiedEnum};
use nuts::{nut04::MintQuoteState, nut05::MeltQuoteState};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::{Code, transport::Channel};
use tracing::{Level, error, event};
use uuid::Uuid;

use crate::{
    db::{self, mint_quote::PendingMintQuote, wad::SyncData},
    errors::Error,
    mint,
    wallet::SeedPhraseManager,
};

#[derive(Debug, thiserror::Error)]
pub enum SyncMintQuotesError {
    #[error("failed to sync mint quote with id {0}: {1}")]
    SyncOne(String, SyncMintQuoteError),
}

#[derive(Debug, Default, Clone)]
pub struct MintQuotesStateUpdate {
    pub deleted: Vec<String>,
    pub unchanged: Vec<(String, MintQuoteState)>,
    pub changed: Vec<(String, MintQuoteState)>,
}

pub async fn mint_quotes(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    pending_mint_quotes: Vec<PendingMintQuote>,
) -> Result<MintQuotesStateUpdate, SyncMintQuotesError> {
    let mut states_updates = MintQuotesStateUpdate::default();
    for pending_mint_quote in pending_mint_quotes {
        let new_state = {
            match mint_quote(
                pool.clone(),
                node_client,
                pending_mint_quote.method.clone(),
                pending_mint_quote.id.clone(),
            )
            .await
            {
                Ok(opt) => match opt {
                    Some(new_state) => new_state,
                    None => {
                        states_updates.deleted.push(pending_mint_quote.id);
                        continue;
                    }
                },
                Err(e) => return Err(SyncMintQuotesError::SyncOne(pending_mint_quote.id, e)),
            }
        };

        if new_state == MintQuoteState::Paid {
            event!(name: "mint-quote-paid",  Level::INFO, quote_id=pending_mint_quote.id);
            if let Err(e) = mint::redeem_quote(
                seed_phrase_manager.clone(),
                pool.clone(),
                node_client,
                pending_mint_quote.method,
                &pending_mint_quote.id,
                node_id,
                &pending_mint_quote.unit,
                pending_mint_quote.amount,
            )
            .await
            {
                error!(
                    "Failed to redeem mint quote {}: {}",
                    pending_mint_quote.id, e
                );
            } else {
                event!(name: "mint-quote-redeemed", Level::INFO, quote_id=pending_mint_quote.id);
            }
        }

        if new_state == pending_mint_quote.state {
            states_updates
                .unchanged
                .push((pending_mint_quote.id, new_state));
        } else {
            states_updates
                .changed
                .push((pending_mint_quote.id, new_state));
        }
    }

    Ok(states_updates)
}

#[derive(Debug, thiserror::Error)]
pub enum SyncMintQuoteError {
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
    #[error("failed to intact witht he node: {0}")]
    Tonic(#[from] tonic::Status),
}

/// Sync the state of this quote from the node.
///
/// 1. query the node for the state
/// 2. delete expired quote
/// 3. update state in database
///
/// Returns node if the mint quote has been deleted,
/// otherwise returns its current state.
pub async fn mint_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<Option<MintQuoteState>, SyncMintQuoteError> {
    let response = node_client
        .mint_quote_state(QuoteStateRequest {
            method,
            quote: quote_id.clone(),
        })
        .await;

    let db_conn = pool.get()?;
    match response {
        Ok(response) => {
            let response = response.into_inner();
            let state = MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(response.state)
                    .map_err(|e| SyncMintQuoteError::InvalidState(e.to_string()))?,
            )?;

            if state == MintQuoteState::Unpaid {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now >= response.expiry {
                    db::mint_quote::delete(&db_conn, &quote_id)
                        .map_err(SyncMintQuoteError::Delete)?;
                    return Ok(None);
                }
            }

            db::mint_quote::set_state(&db_conn, &response.quote, state)
                .map_err(SyncMintQuoteError::SetState)?;

            Ok(Some(state))
        }
        Err(s) if s.code() == Code::NotFound => {
            db::mint_quote::delete(&db_conn, &quote_id).map_err(SyncMintQuoteError::Delete)?;
            Ok(None)
        }
        Err(e) => Err(e)?,
    }
}

pub async fn melt_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<Option<(MeltQuoteState, Vec<String>)>, Error> {
    let response = node_client
        .melt_quote_state(node_client::MeltQuoteStateRequest {
            method,
            quote: quote_id.clone(),
        })
        .await;

    let mut db_conn = pool.get()?;
    match response {
        Err(status) if status.code() == tonic::Code::DeadlineExceeded => {
            db::melt_quote::delete(&db_conn, &quote_id)?;
            Ok(None)
        }
        Ok(response) => {
            let response = response.into_inner();
            let state =
                MeltQuoteState::try_from(node_client::MeltQuoteState::try_from(response.state)?)?;

            let tx = db_conn.transaction()?;
            match state {
                MeltQuoteState::Unpaid => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now >= response.expiry {
                        db::melt_quote::delete(&tx, &quote_id)?;
                        return Ok(None);
                    }
                }
                MeltQuoteState::Pending => {}
                MeltQuoteState::Paid => {
                    if !response.transfer_ids.is_empty() {
                        let transfer_ids_to_store = serde_json::to_string(&response.transfer_ids)?;
                        db::melt_quote::register_transfer_ids(
                            &tx,
                            &quote_id,
                            &transfer_ids_to_store,
                        )?;
                    }
                }
            }

            db::melt_quote::update_state(&tx, &quote_id, response.state)?;
            tx.commit()?;

            Ok(Some((state, response.transfer_ids)))
        }
        Err(e) => Err(e)?,
    }
}

pub async fn pending_wads(
    pool: Pool<SqliteConnectionManager>,
    root_ca_certificate: Option<tonic::transport::Certificate>,
) -> Result<Vec<WadSyncResult>, Error> {
    let pending_wads = {
        let db_conn = pool.get()?;
        db::wad::get_pending_wads(&db_conn)?
    };

    let mut results = Vec::with_capacity(pending_wads.len());
    for sync_data in pending_wads {
        let wad_id = sync_data.id;
        let result = sync_single_wad(pool.clone(), sync_data, root_ca_certificate.clone()).await;

        results.push(WadSyncResult {
            wad_id,
            result: result.map_err(|e| e.to_string()),
        });
    }

    Ok(results)
}

async fn sync_single_wad(
    pool: Pool<SqliteConnectionManager>,
    sync_info: SyncData,
    root_ca_certificate: Option<tonic::transport::Certificate>,
) -> Result<Option<db::wad::WadStatus>, Error> {
    use node_client::{CheckStateRequest, ProofState};

    let SyncData {
        id: wad_id,
        r#type: _wad_type,
        node_url,
    } = sync_info;

    let proof_ys = {
        let db_conn = pool.get()?;
        db::wad::get_proofs_ys_by_id(&db_conn, wad_id)?
    };

    if proof_ys.is_empty() {
        return Ok(None);
    }

    let mut node_client = crate::connect_to_node(&node_url, root_ca_certificate).await?;

    let check_request = CheckStateRequest {
        ys: proof_ys.iter().map(|y| y.to_bytes().to_vec()).collect(),
    };

    let response = node_client.check_state(check_request).await?;
    let states = response.into_inner().states;
    let all_spent = states
        .iter()
        .all(|state| match ProofState::try_from(state.state) {
            Ok(ProofState::PsSpent) => true,
            Ok(ProofState::PsUnspent | ProofState::PsPending) => false,
            Ok(_unexpected_state) => false,
            Err(_) => false,
        });

    for state in &states {
        ProofState::try_from(state.state).map_err(|_| {
            Error::UnexpectedProofState(format!(
                "Invalid proof state encountered for WAD {}: {:?}",
                wad_id, state.state
            ))
        })?;
    }

    if all_spent {
        let db_conn = pool.get()?;
        db::wad::update_wad_status(&db_conn, wad_id, db::wad::WadStatus::Finished)?;
        Ok(Some(db::wad::WadStatus::Finished))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct WadSyncResult {
    pub wad_id: Uuid,
    pub result: Result<Option<db::wad::WadStatus>, String>,
}
