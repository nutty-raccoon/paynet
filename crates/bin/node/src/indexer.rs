// use prost::Message;
use futures::TryStreamExt;
use sqlx::PgConnection;
use starknet_payment_indexer::{ApibaraIndexerService, PaymentEvent, Message};
use starknet_types_core::felt::Felt;
use tracing::{debug, info};
use uuid::Uuid;
use crate::errors::{InitializationError, ServiceError};
use nuts::nut04::MintQuoteState;
use starknet_types::Unit::Strk;

pub async fn init_indexer_task(
    apibara_token: String,
    strk_token_address: Felt,
    recipient_address: Felt,
) -> Result<ApibaraIndexerService, InitializationError> {
    let conn = rusqlite::Connection::open_in_memory().map_err(InitializationError::OpenSqlite)?;

    let service = starknet_payment_indexer::ApibaraIndexerService::init(
        conn,
        apibara_token,
        vec![(recipient_address, strk_token_address)],
    )
    .await
    .map_err(InitializationError::InitIndexer)?;

    Ok(service)
}

pub async fn listen_to_indexer(
    db_conn: &mut PgConnection,
    mut indexer_service: ApibaraIndexerService,
) -> Result<(), ServiceError> {
    info!("Listening indexer events");
    
    while let Some(event) = indexer_service
        .try_next()
        .await
        .map_err(ServiceError::Indexer)?
    {
        match event.clone() {
            Message::Payment(payment_events) => {
                for payment_event in payment_events {
                    db_node::insert_new_payment_event(
                        &payment_event
                    );
                    let amt = sqlx::query!(
                        "SELECT SUM(amount) as total_amount
                        FROM payment_event
                        WHERE invoice_id = $1"  
                    )
                    .bind(payment_event.invoice_id)
                    .fetch_one(&db_conn)
                    .await?;
                    let amount = sqlx::query!(
                        "SELECT amount FROM mint_quote WHERE invoice = ? LIMIT 1"
                    )
                    .bind(payment_event.invoice_id)
                    .fetch_one(&db_conn)
                    .await?;
                    if Strk.convert_u256_into_amount(amt) >= Strk.convert_u256_into_amount(amount):
                        break
                }
            },  
            Message::Invalidate { 
                last_valid_block_number,
                last_valid_block_hash
            } => {
                todo!();
            }
        }

        debug!("Event received:\n{:?}", event);
    }
    db_node::mint_quote::set_state(db_conn, quote_id, MintQuoteState::Paid);

    Ok(())
}
