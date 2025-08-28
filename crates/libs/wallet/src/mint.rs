use node_client::{
    MintQuoteRequest, MintQuoteResponse, MintRequest, NodeClient, hash_mint_request,
};
use nuts::{Amount, SplitTarget, nut04::MintQuoteState, nut19::Route, traits::Unit};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;

use crate::{
    acknowledge, db,
    errors::{Error, handle_out_of_sync_keyset_errors},
    node::refresh_keysets,
    sync,
    types::{BlindingData, PreMints},
    wallet::SeedPhraseManager,
};

pub async fn create_quote<U: Unit>(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    method: String,
    amount: Amount,
    unit: U,
) -> Result<MintQuoteResponse, Error> {
    let response = node_client
        .mint_quote(MintQuoteRequest {
            method: method.clone(),
            amount: amount.into(),
            unit: unit.as_ref().to_string(),
            description: None,
        })
        .await?
        .into_inner();

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
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<QuotePaymentIssue, Error> {
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

#[allow(clippy::too_many_arguments)]
pub async fn redeem_quote(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
    node_id: u32,
    unit: &str,
    total_amount: Amount,
) -> Result<(), Error> {
    refresh_keysets(pool.clone(), node_client, node_id).await?;

    let blinding_data = {
        let db_conn = pool.get()?;
        BlindingData::load_from_db(seed_phrase_manager, &db_conn, node_id, unit)?
    };

    let pre_mints = PreMints::generate_for_amount(total_amount, &SplitTarget::None, blinding_data)?;

    let outputs = pre_mints.build_node_client_outputs();

    let mint_request = MintRequest {
        method,
        quote: quote_id.clone(),
        outputs,
    };

    let mint_request_hash = hash_mint_request(&mint_request);

    let mint_result = node_client.mint(mint_request).await;
    let mint_response = match mint_result {
        Ok(r) => r.into_inner(),
        Err(e) => {
            // TODO: add retry once we are sync
            handle_out_of_sync_keyset_errors(&e, pool, node_client, node_id).await?;
            return Err(e.into());
        }
    };

    {
        let mut db_conn = pool.get()?;
        let tx = db_conn.transaction()?;
        pre_mints.store_new_tokens(&tx, node_id, mint_response.signatures)?;
        db::mint_quote::set_state(&tx, &quote_id, MintQuoteState::Issued)?;
        tx.commit()?;
    }

    acknowledge(node_client, Route::Mint, mint_request_hash).await?;

    Ok(())
}
