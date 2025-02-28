use crate::Error;
use crate::errors::{InitializationError, ServiceError};
use futures::TryStreamExt;
use nuts::Amount;
use nuts::nut04::MintQuoteState;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use starknet_payment_indexer::{ApibaraIndexerService, Message, PaymentEvent};
use starknet_types::{StarknetU256, Unit::Strk};
use starknet_types_core::felt::Felt;
use std::str::FromStr;
use tracing::info;

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
    mut db_conn: PoolConnection<Postgres>,
    mut indexer_service: ApibaraIndexerService,
) -> Result<(), crate::errors::Error> {
    info!("Listening indexer events");

    while let Some(event) = indexer_service
        .try_next()
        .await
        .map_err(ServiceError::Indexer)?
    {
        match event {
            Message::Payment(payment_events) => {
                process_payment_event(payment_events, &mut db_conn).await?;
            }
            Message::Invalidate {
                last_valid_block_number: _,
                last_valid_block_hash: _,
            } => {
                todo!();
            }
        }
    }

    Ok(())
}

async fn process_payment_event(
    payment_events: Vec<PaymentEvent>,
    db_conn: &mut PgConnection,
) -> Result<(), Error> {
    for payment_event in payment_events {
        let quote_id = match db_node::mint_quote::get_quote_id_by_invoice_id(
            db_conn,
            payment_event.invoice_id.to_string(),
        )
        .await?
        {
            None => continue,
            Some(mint_quote_id) => mint_quote_id,
        };
        db_node::payment_event::insert_new_payment_event(db_conn, &payment_event).await?;
        let current_paid =
            db_node::payment_event::get_current_paid(db_conn, payment_event.invoice_id.to_string())
                .await?
                .map(|(low, high)| -> Result<primitive_types::U256, Error> {
                    let amount_as_strk_256 = StarknetU256 {
                        low: Felt::from_str(&low).map_err(|e| ServiceError::Indexer(e.into()))?,
                        high: Felt::from_str(&high).map_err(|e| ServiceError::Indexer(e.into()))?,
                    };

                    Ok(primitive_types::U256::from(amount_as_strk_256))
                })
                .try_fold(primitive_types::U256::zero(), |acc, a| match a {
                    Ok(v) => v.checked_add(acc).ok_or(Error::Overflow),
                    Err(e) => Err(e),
                })?;

        let quote_expected_amount = db_node::mint_quote::get_amount_from_invoice_id(
            db_conn,
            payment_event.invoice_id.to_string(),
        )
        .await?;

        let current_paid_starknet_u256: StarknetU256 = current_paid.into();

        let current_paid_amount = match Strk.convert_u256_into_amount(current_paid_starknet_u256) {
            Ok((amount, _remainder)) => amount,
            Err(e) => return Err(Error::Starknet(e)),
        };

        if current_paid_amount >= Amount::from(quote_expected_amount) {
            db_node::mint_quote::set_state(db_conn, quote_id, MintQuoteState::Paid).await?;
        }
    }

    Ok(())
}
