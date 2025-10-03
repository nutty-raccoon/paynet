use std::time::{SystemTime, UNIX_EPOCH};

use cashu_client::CashuClient;
use node_client::UnspecifiedEnum;
use nuts::nut04::MintQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::Code;
use tracing::{Level, error, event};

use crate::{
    db::{self, mint_quote::PendingMintQuote},
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
    pub unchanged: Vec<PendingMintQuote>,
    pub changed: Vec<PendingMintQuote>,
}

pub async fn mint_quotes(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    node_id: u32,
    pending_mint_quotes: Vec<PendingMintQuote>,
    trigger_redeem: bool,
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

        let new_state = if trigger_redeem && new_state == MintQuoteState::Paid {
            event!(name: "mint-quote-paid",  Level::INFO, quote_id=pending_mint_quote.id, "Mint quote paid");
            if let Err(error) = mint::redeem_quote(
                seed_phrase_manager.clone(),
                pool.clone(),
                node_client,
                pending_mint_quote.method.clone(),
                &pending_mint_quote.id,
                node_id,
                &pending_mint_quote.unit,
                pending_mint_quote.amount,
            )
            .await
            {
                event!(name: "mint-quote-redeemed", Level::ERROR, quote_id=pending_mint_quote.id, %error, "Failed to redeem mint quote");
                new_state
            } else {
                event!(name: "mint-quote-redeemed", Level::INFO, quote_id=pending_mint_quote.id, "Deposit finalized");
                MintQuoteState::Issued
            }
        } else {
            new_state
        };

        if new_state == pending_mint_quote.state {
            states_updates.unchanged.push(pending_mint_quote);
        } else {
            let mut pending_mint_quote = pending_mint_quote;
            pending_mint_quote.state = new_state;
            states_updates.changed.push(pending_mint_quote);
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
    Tonic(#[from] cashu_client::Error),
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
    node_client: &mut impl CashuClient,
    method: String,
    quote_id: String,
) -> Result<Option<MintQuoteState>, SyncMintQuoteError> {
    let response = node_client.mint_quote_state(method, quote_id.clone()).await;

    let db_conn = pool.get()?;
    match response {
        Ok(response) => {
            let state =
                MintQuoteState::try_from(node_client::MintQuoteState::from(response.state))?;

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
        Err(cashu_client::Error::Grpc(s)) if s.code() == Code::NotFound => {
            db::mint_quote::delete(&db_conn, &quote_id).map_err(SyncMintQuoteError::Delete)?;
            Ok(None)
        }
        Err(e) => Err(e)?,
    }
}
