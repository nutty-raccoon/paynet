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

    let service =
        starknet_payment_indexer::ApibaraIndexerService::init(conn, apibara_token, vec![(
            recipient_address,
            strk_token_address,
        )])
        .await
        .map_err(InitializationError::InitIndexer)?;

    Ok(service)
}

pub async fn listen_to_indexer(
    db_conn: &mut PgConnection,
    mut indexer_service: ApibaraIndexerService,
) -> Result<(), ServiceError> {
    info!("Listening indexer events");

    let mut current_amount:u64 = 0;
    
    while let Some(event) = indexer_service
        .try_next()
        .await
        .map_err(ServiceError::Indexer)?
    {
        let tx = sqrlx::query_as!(
            invoice,
            r#"
            SELECT invoice_id
            FROM invoices
            WHERE invoice_id = $1
            "#,
            invoice_id
        )
        .fetch_optional(db_conn)
        .await;

        match event.clone() {
            Message::Payment(payment_events) => {
                for payment_event in payment_events {
                    let converted_amount = Strk.convert_u256_into_amount(payment_event.amount)?;
                    current_amount += converted_amount;
                    if current_amount >= tx.amount:
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
        //aqui eu vou pegar o evento
        debug!("Event received:\n{:?}", event);
    }
    db_node::mint_quote::set_state(db_conn, tx.quote_id, MintQuoteState::Paid);

    Ok(())
}
