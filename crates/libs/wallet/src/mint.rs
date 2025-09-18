use cashu_client::{CashuClient, ClientMintQuoteRequest, GrpcClient};
use node_client::{MintRequest, NodeClient, hash_mint_request};
use nuts::{
    Amount, SplitTarget,
    nut04::{MintQuoteResponse, MintQuoteState},
    nut19::Route,
    traits::Unit,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;

use crate::{
    acknowledge, db,
    errors::Error,
    node::refresh_keysets,
    sync::{self, SyncMintQuoteError},
    types::{BlindingData, PreMints},
    wallet::SeedPhraseManager,
};

pub async fn create_quote<U: Unit>(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    node_id: u32,
    method: String,
    amount: Amount,
    unit: U,
) -> Result<MintQuoteResponse<String>, Error> {
    let response = node_client
        .mint_quote(ClientMintQuoteRequest {
            method: method.clone(),
            amount: amount.into(),
            unit: unit.as_ref().to_string(),
            description: None,
        })
        .await?;

    let db_conn = pool.get()?;
    db::mint_quote::store(&db_conn, node_id, method, amount, unit.as_ref(), &response)?;

    Ok(response)
}

pub enum QuotePaymentIssue {
    Paid,
    Expired,
}

pub async fn wait_for_quote_payment(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    method: String,
    quote_id: String,
) -> Result<QuotePaymentIssue, SyncMintQuoteError> {
    loop {
        let state =
            match sync::mint_quote(pool.clone(), node_client, method.clone(), quote_id.clone())
                .await?
            {
                Some(new_state) => new_state,
                None => {
                    return Ok(QuotePaymentIssue::Expired);
                }
            };

        if state == MintQuoteState::Paid {
            return Ok(QuotePaymentIssue::Paid);
        }

        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedeemQuoteError {
    #[error("failed to refresh keyset: {0}")]
    RefreshKeyset(#[from] crate::node::RefreshNodeKeysetError),
    #[error(transparent)]
    R2d2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] crate::wallet::Error),
    #[error(transparent)]
    Grpc(#[from] tonic::Status),
    #[error("failed to generate pre-mints: {0}")]
    PreMints(#[from] crate::errors::Error),
    #[error(transparent)]
    CashuClient(#[from] cashu_client::Error),
}

#[allow(clippy::too_many_arguments)]
pub async fn redeem_quote(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    method: String,
    quote_id: &str,
    node_id: u32,
    unit: &str,
    total_amount: Amount,
) -> Result<(), RedeemQuoteError> {
    refresh_keysets(pool.clone(), node_client, node_id).await?;

    let blinding_data = {
        let db_conn = pool.get()?;
        BlindingData::load_from_db(seed_phrase_manager, &db_conn, node_id, unit)?
    };

    let pre_mints = PreMints::generate_for_amount(total_amount, &SplitTarget::None, blinding_data)?;

    let outputs = pre_mints.build_nuts_outputs();

    let mint_request = nuts::nut04::MintRequest {
        quote: quote_id.to_string(),
        outputs,
    };

    let mint_request_hash = nuts::nut19::hash_mint_request(&mint_request);

    let mint_result = node_client.mint(mint_request, method).await;
    let mint_response = match mint_result {
        Ok(r) => r,
        Err(e) => {
            // TODO: add retry once we are sync
            if node_client.keyset_refresh(&e) {
                crate::node::refresh_keysets(pool, node_client, node_id).await?;
            }
            return Err(e.into());
        }
    };

    {
        let mut db_conn = pool.get()?;
        let tx = db_conn.transaction()?;
        pre_mints.store_new_tokens(&tx, node_id, mint_response.signatures)?;
        db::mint_quote::set_state(&tx, quote_id, MintQuoteState::Issued)?;
        tx.commit()?;
    }

    acknowledge(node_client, Route::Mint, mint_request_hash).await?;

    Ok(())
}
