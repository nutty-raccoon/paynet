use std::time::{SystemTime, UNIX_EPOCH};

use node_client::{
    MintQuoteRequest, MintQuoteResponse, MintRequest, NodeClient, QuoteStateRequest,
    hash_mint_request,
};
use nuts::{Amount, SplitTarget, nut04::MintQuoteState, nut19::Route, traits::Unit};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use tonic::transport::Channel;

use crate::{
    acknowledge, db,
    errors::Error,
    types::{BlindingData, PreMints},
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
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<QuotePaymentIssue, Error> {
    loop {
        let state =
            match get_quote_state(db_conn, node_client, method.clone(), quote_id.clone()).await? {
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

pub async fn get_quote_state(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<Option<MintQuoteState>, Error> {
    let response = node_client
        .mint_quote_state(QuoteStateRequest {
            method,
            quote: quote_id.clone(),
        })
        .await;

    match response {
        Err(status) if status.code() == tonic::Code::DeadlineExceeded => {
            db::mint_quote::delete(db_conn, &quote_id)?;
            Ok(None)
        }
        Ok(response) => {
            let response = response.into_inner();
            let state = MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(response.state)
                    .map_err(|e| Error::Conversion(e.to_string()))?,
            )?;

            if state == MintQuoteState::Unpaid {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now >= response.expiry {
                    db::mint_quote::delete(db_conn, &quote_id)?;
                    return Ok(None);
                }
            }

            db::mint_quote::set_state(db_conn, &response.quote, state)?;

            Ok(Some(state))
        }
        Err(e) => Err(e)?,
    }
}

pub async fn redeem_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
    node_id: u32,
    unit: &str,
    total_amount: Amount,
) -> Result<(), Error> {
    let blinding_data = {
        let db_conn = pool.get()?;
        BlindingData::load_from_db(&db_conn, node_id, unit)?
    };

    let pre_mints = PreMints::generate_for_amount(total_amount, &SplitTarget::None, blinding_data)?;

    let outputs = pre_mints.build_node_client_outputs();

    let mint_request = MintRequest {
        method,
        quote: quote_id.clone(),
        outputs,
    };

    let mint_request_hash = hash_mint_request(&mint_request);
    let mint_response = node_client.mint(mint_request).await?.into_inner();

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
