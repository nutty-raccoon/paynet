use cashu_client::{CashuClient, ClientMeltQuoteRequest, ClientMeltQuoteResponse};
use nuts::Amount;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::{
    acknowledge, db,
    errors::{Error, handle_already_spent_proofs, handle_crypto_invalid_proofs},
    fetch_inputs_ids_from_db_or_node, sync,
    types::ProofState,
    unprotected_load_tokens_from_db,
    wallet::SeedPhraseManager,
};

pub async fn create_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    node_id: u32,
    method: String,
    unit: String,
    request: String,
) -> Result<ClientMeltQuoteResponse, Error> {
    let response = node_client
        .melt_quote(ClientMeltQuoteRequest {
            method: method.clone(),
            unit,
            request: request.clone(),
        })
        .await?;

    let db_conn = pool.get()?;
    db::melt_quote::store(&db_conn, node_id, method, request, &response)?;

    Ok(response)
}

#[derive(Debug, thiserror::Error)]
pub enum PayMeltQuoteError {
    #[error("failed get db connection from pool: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("failed to load the proofs ids from db or node: {0}")]
    FetchInputsIds(#[source] Error),
    #[error("not enough funds")]
    NotEnoughFunds,
    #[error("failed to start database transaction: {0}")]
    StartDbTransaction(#[source] rusqlite::Error),
    #[error("failed to commit database transaction: {0}")]
    CommitDbTransaction(#[source] rusqlite::Error),
    #[error("failed to load the proofs from db: {0}")]
    LoadTokens(#[source] rusqlite::Error),
    #[error(transparent)]
    SetProofsState(#[from] db::proof::SetProofsToStateError),
    #[error("failed update quote state: {0}")]
    UpdateQuoteState(#[source] rusqlite::Error),
    #[error("failed to register transfers ids : {0}")]
    RegisterTransfersIds(#[source] rusqlite::Error),
    #[error("failed to handle the error occured during proof verification: {0}")]
    HandleProofsVerficationErrors(#[source] Error),
    #[error("melt operation failed: {0}")]
    MeltOperationFailed(#[source] cashu_client::Error),
    #[error("failed to acknowledge: {0}")]
    Acknowledge(#[source] cashu_client::Error),
    #[error("failed to serialize transfer ids: {0}")]
    SerializeTransferIds(#[from] serde_json::Error),
    #[error(transparent)]
    UnprotectedLoadTokensFormDb(#[from] crate::UnprotectedLoadTokensFormDbError),
    #[error(transparent)]
    SyncMeltQuote(#[from] sync::SyncMeltQuoteError),
}

#[allow(clippy::too_many_arguments)]
pub async fn pay_quote(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    node_id: u32,
    quote_id: String,
    amount: Amount,
    method: String,
    unit: &str,
) -> Result<nuts::nut05::MeltResponse, PayMeltQuoteError> {
    // Gather the proofs
    let proofs_ids = fetch_inputs_ids_from_db_or_node(
        seed_phrase_manager,
        pool.clone(),
        node_client,
        node_id,
        amount,
        unit,
    )
    .await
    .map_err(PayMeltQuoteError::FetchInputsIds)?
    .ok_or(PayMeltQuoteError::NotEnoughFunds)?;
    let inputs = {
        let db_conn = pool.get()?;
        unprotected_load_tokens_from_db(&db_conn, &proofs_ids)?
    };

    // Create melt request
    let melt_request = nuts::nut05::MeltRequest {
        quote: quote_id.clone(),
        inputs,
    };

    let melt_request_hash = nuts::nut19::hash_melt_request(&melt_request);

    let melt_res = node_client.melt(method, melt_request).await;
    // If this fail we won't be able to actualize the proof state. Which may lead to some bugs.
    let mut db_conn = pool.get()?;

    // Call the node and handle failure
    let melt_response = match melt_res {
        Ok(r) => r,
        Err(e) => {
            if let Some(errors) = node_client.extract_proof_errors(&e) {
                if !errors[0].index.is_empty() {
                    handle_already_spent_proofs(errors[0].index.clone(), &proofs_ids, &db_conn)
                        .map_err(PayMeltQuoteError::HandleProofsVerficationErrors)?;
                }
                if !errors[1].index.is_empty() {
                    handle_crypto_invalid_proofs(errors[1].index.clone(), &proofs_ids, &db_conn)
                        .map_err(PayMeltQuoteError::HandleProofsVerficationErrors)?;
                }
            }
            return Err(PayMeltQuoteError::MeltOperationFailed(e));
        }
    };

    // Register the consumption of our proofs
    db::proof::set_proofs_to_state(&db_conn, &proofs_ids, ProofState::Spent)?;

    // Relieve the node cache once we receive the answer
    acknowledge(node_client, nuts::nut19::Route::Melt, melt_request_hash)
        .await
        .map_err(PayMeltQuoteError::Acknowledge)?;

    if melt_response.state == nuts::nut05::MeltQuoteState::Paid {
        let tx = db_conn
            .transaction()
            .map_err(PayMeltQuoteError::StartDbTransaction)?;
        db::melt_quote::set_state(&tx, &quote_id, melt_response.state)
            .map_err(PayMeltQuoteError::UpdateQuoteState)?;
        if melt_response.transfer_ids.is_some() {
            let transfer_ids_to_store = serde_json::to_string(&melt_response.transfer_ids)?;
            db::melt_quote::register_transfer_ids(&tx, &quote_id, &transfer_ids_to_store)
                .map_err(PayMeltQuoteError::RegisterTransfersIds)?;
        }
        tx.commit()
            .map_err(PayMeltQuoteError::CommitDbTransaction)?;
    }

    Ok(melt_response)
}

pub async fn wait_for_payment(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut impl CashuClient,
    method: String,
    quote_id: String,
) -> Result<Option<Vec<String>>, PayMeltQuoteError> {
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

pub fn format_melt_transfers_id_into_term_message(transfer_ids: Vec<String>) -> String {
    let mut string_to_print = "Melt done. Withdrawal settled with tx".to_string();
    if transfer_ids.len() != 1 {
        string_to_print.push('s');
    }
    string_to_print.push_str(": ");
    let mut iterator = transfer_ids.into_iter();
    string_to_print.push_str(&iterator.next().unwrap());
    for tx_hash in iterator {
        string_to_print.push_str(", ");
        string_to_print.push_str(&tx_hash);
    }

    string_to_print
}
