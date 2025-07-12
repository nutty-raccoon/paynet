use node_client::{
    MeltQuoteRequest, MeltQuoteResponse, MeltQuoteState, MeltResponse, NodeClient,
    hash_melt_request,
};
use nuts::{Amount, traits::Unit};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;

use crate::{
    acknowledge, convert_inputs, db, errors::Error, fetch_inputs_ids_from_db_or_node,
    load_tokens_from_db, sync, types::ProofState,
};

pub async fn create_quote<U: Unit>(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    method: String,
    unit: U,
    request: String,
) -> Result<MeltQuoteResponse, Error> {
    let response = node_client
        .melt_quote(MeltQuoteRequest {
            method: method.clone(),
            unit: unit.to_string(),
            request: request.clone(),
        })
        .await?
        .into_inner();

    let db_conn = pool.get()?;
    db::melt_quote::store(&db_conn, node_id, method, request, &response)?;

    Ok(response)
}

pub async fn pay_quote<U: Unit>(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    quote_id: String,
    amount: Amount,
    method: String,
    unit: U,
) -> Result<MeltResponse, Error> {
    // Gather the proofs
    let proofs_ids =
        fetch_inputs_ids_from_db_or_node(pool.clone(), node_client, node_id, amount, unit)
            .await?
            .ok_or(Error::NotEnoughFunds)?;
    let inputs = load_tokens_from_db(&*pool.get()?, &proofs_ids)?;

    // Create melt request
    let melt_request = node_client::MeltRequest {
        method: method.clone(),
        quote: quote_id.clone(),
        inputs: convert_inputs(&inputs),
    };

    let melt_request_hash = hash_melt_request(&melt_request);

    let melt_res = node_client.melt(melt_request).await;
    // If this fail we won't be able to actualize the proof state. Which may lead to some bugs.
    let mut db_conn = pool.get()?;

    // Call the node and handle failure
    let melt_response = match melt_res {
        Ok(r) => r.into_inner(),
        Err(e) => {
            // Reset the proof state
            // TODO: if the error is due to one of the proof being already spent, we should be removing those from db
            // in order to not use them in the future
            db::proof::set_proofs_to_state(&db_conn, &proofs_ids, ProofState::Unspent)?;
            return Err(e.into());
        }
    };

    // Register the consumption of our proofs
    db::proof::set_proofs_to_state(&db_conn, &proofs_ids, ProofState::Spent)?;

    // Relieve the node cache once we receive the answer
    acknowledge(node_client, nuts::nut19::Route::Melt, melt_request_hash).await?;

    if melt_response.state == MeltQuoteState::MlqsPaid as i32 {
        let tx = db_conn.transaction()?;
        db::melt_quote::update_state(&tx, &quote_id, melt_response.state)?;
        if !melt_response.transfer_ids.is_empty() {
            let transfer_ids_to_store = serde_json::to_string(&melt_response.transfer_ids)?;
            db::melt_quote::register_transfer_ids(&tx, &quote_id, &transfer_ids_to_store)?;
        }
        tx.commit()?;
    }

    Ok(melt_response)
}

pub async fn wait_for_payment(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<Option<Vec<String>>, Error> {
    loop {
        let quote_state =
            sync::melt_quote(pool.clone(), node_client, method.clone(), quote_id.clone()).await?;

        match quote_state {
            Some((nuts::nut05::MeltQuoteState::Paid, tx_ids)) => return Ok(Some(tx_ids)),
            None => return Ok(None),
            _ => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
        }
    }
}
