// use prost::Message;
use crate::Error;
use crate::errors::{InitializationError, ServiceError};
use futures::TryStreamExt;
use nuts::nut04::MintQuoteState;
use sqlx::PgConnection;
use starknet_payment_indexer::{ApibaraIndexerService, Message};
use starknet_types::{StarknetU256, Unit::Strk};
use starknet_types_core::felt::Felt;
use std::str::FromStr;
use tracing::{debug, info};

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
) -> Result<(), crate::errors::Error> {
    info!("Listening indexer events");

    while let Some(event) = indexer_service
        .try_next()
        .await
        .map_err(ServiceError::Indexer)?
    {
        match event.clone() {
            Message::Payment(payment_events) => {
                for payment_event in payment_events {
                    let mqid = match db_node::mint_quote::get_quote_id_by_invoice_id(
                        db_conn,
                        payment_event.invoice_id.to_string(),
                    )
                    .await?
                    {
                        None => {
                            continue;
                        }
                        Some(mint_quote_id) => mint_quote_id,
                    };
                    db_node::payment_event::insert_new_payment_event(db_conn, &payment_event)
                        .await?;
                    let current_paid = sqlx::query!(
                        r#"SELECT amount_high, amount_low
                        FROM payment_event
                        WHERE invoice_id = $1"#,
                        payment_event.invoice_id.to_string()
                    )
                    .fetch_all(&mut *db_conn)
                    .await?
                    .iter()
                    .map(|r| -> Result<primitive_types::U256, Error> {
                        let strk_256 = StarknetU256 {
                            low: Felt::from_str(&r.amount_low)
                                .map_err(|e| ServiceError::Indexer(e.into()))?,
                            high: Felt::from_str(&r.amount_high)
                                .map_err(|e| ServiceError::Indexer(e.into()))?,
                        };

                        Ok(primitive_types::U256::from(strk_256))
                    })
                    .try_fold(
                        primitive_types::U256::zero(),
                        |acc, x| match x {
                            Ok(v) => v.checked_add(acc).ok_or(Error::Overflow),
                            Err(e) => Err(e),
                        },
                    )?;

                    let total_amount = sqlx::query!(
                        r#"SELECT amount FROM mint_quote WHERE invoice = $1 LIMIT 1"#,
                        payment_event.invoice_id.to_string()
                    )
                    .fetch_one(&mut *db_conn)
                    .await?
                    .amount;

                    let current_paid_starknet_u256: StarknetU256 = current_paid.into();

                    let current_paid_amount =
                        match Strk.convert_u256_into_amount(current_paid_starknet_u256) {
                            Ok((amt, _u256)) => amt,
                            Err(_e) => return Err(Error::Overflow),
                        };

                    let total_amount = u64::from_be_bytes(total_amount.to_be_bytes()).into();

                    if current_paid_amount >= total_amount {
                        db_node::mint_quote::set_state(db_conn, mqid, MintQuoteState::Paid).await?;
                        break;
                    }
                }
            }
            Message::Invalidate {
                last_valid_block_number: _,
                last_valid_block_hash: _,
            } => {
                todo!();
            }
        }

        debug!("Event received:\n{:?}", event);
    }

    Ok(())
}
