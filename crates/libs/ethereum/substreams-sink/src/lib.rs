use std::{
    env::{self, VarError},
    sync::Arc,
};

use crate::pb::{invoice_contract::v1::RemittanceEvents, sf::substreams::rpc::v2::BlockScopedData};
use anyhow::{Error, Result, anyhow, format_err};
use db_node::PaymentEvent;
use futures::StreamExt;
use http::Uri;
use nuts::{Amount, nut04::MintQuoteState, nut05::MeltQuoteState};
use prost::Message;
use sqlx::{
    PgConnection, PgPool,
    types::{
        Uuid,
        chrono::{DateTime, Utc},
    },
};
use ethereum_types::{ChainId, Unit, constants::ON_CHAIN_CONSTANTS};
use substreams::SubstreamsEndpoint;
use substreams_stream::{BlockResponse, SubstreamsStream};
use tracing::{Level, debug, error, event};

mod parse_inputs;
#[allow(clippy::enum_variant_names)]
mod pb;
mod substreams;
mod substreams_stream;

pub async fn launch(
    pg_pool: PgPool,
    endpoint_url: Uri,
    chain_id: ChainId,
    cashier_account_address: String, // Ethereum address as hex string
) -> Result<()> {
    let package = parse_inputs::read_package(vec![])?;

    let token = match env::var("SUBSTREAMS_API_TOKEN") {
        Err(VarError::NotPresent) => None,
        Err(e) => Err(e)?,
        Ok(val) if val.is_empty() => None,
        Ok(val) => Some(val),
    };

    let endpoint = Arc::new(SubstreamsEndpoint::new(endpoint_url, token).await?);

    const OUTPUT_MODULE_NAME: &str = "map_invoice_contract_events";

    let initial_block = package
        .modules
        .as_ref()
        .unwrap()
        .modules
        .iter()
        .find(|m| m.name == OUTPUT_MODULE_NAME)
        .ok_or_else(|| format_err!("module '{}' not found in package", OUTPUT_MODULE_NAME))?
        .initial_block;

    let mut db_conn = pg_pool.acquire().await?;

    let cursor: Option<String> = load_persisted_cursor(&mut db_conn).await?;

    let mut stream = SubstreamsStream::new(
        endpoint,
        cursor,
        package.modules,
        OUTPUT_MODULE_NAME.to_string(),
        initial_block as i64,
        0,
    );

    loop {
        match stream.next().await {
            None => {
                break;
            }
            Some(Ok(BlockResponse::New(data))) => {
                process_block_scoped_data(&mut db_conn, &data, &chain_id, &cashier_account_address)
                    .await?;
                persist_cursor(&mut db_conn, data.cursor).await?;
            }
            Some(Ok(BlockResponse::Undo(undo_signal))) => {
                delete_invalid_blocks(&mut db_conn, undo_signal.last_valid_block.unwrap().number)
                    .await?;
                persist_cursor(&mut db_conn, undo_signal.last_valid_cursor).await?;
            }
            Some(Err(err)) => {
                return Err(err);
            }
        }
    }

    Ok(())
}

async fn process_block_scoped_data(
    conn: &mut PgConnection,
    data: &BlockScopedData,
    chain_id: &ChainId,
    cashier_account_address: &str,
) -> Result<(), Error> {
    let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

    let clock = data.clock.as_ref().unwrap();
    let timestamp = clock.timestamp.as_ref().unwrap();
    let date = DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
        .expect("received timestamp should always be valid");

    sqlx::query(r#"
            INSERT INTO substreams_ethereum_block (id, number, timestamp) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;
        "#)
        .bind(&clock.id)
            .bind(i64::try_from(clock.number).unwrap())
                .bind(date)
    .execute(&mut *conn).await?;

    let events = RemittanceEvents::decode(output.value.as_slice())?;

    println!(
        "Block #{} - Payload {} ({} bytes) - Drift {}s",
        clock.number,
        output.type_url.replace("type.googleapis.com/", ""),
        output.value.len(),
        -date.signed_duration_since(Utc::now()).num_seconds()
    );

    process_payment_event(
        events,
        conn,
        chain_id,
        cashier_account_address,
        clock.id.clone(),
    )
    .await?;

    Ok(())
}

async fn delete_invalid_blocks(
    conn: &mut PgConnection,
    last_valid_block_number: u64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        r#"
            DELETE FROM substreams_ethereum_block WHERE number > $1;
        "#,
    )
    .bind(i64::try_from(last_valid_block_number).unwrap())
    .execute(conn)
    .await?;

    Ok(())
}

async fn persist_cursor(conn: &mut PgConnection, cursor: String) -> Result<(), anyhow::Error> {
    sqlx::query(
        r#"
            INSERT INTO substreams_cursor (name, cursor) VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET cursor = excluded.cursor
        "#,
    )
    .bind("ethereum")
    .bind(cursor)
    .execute(conn)
    .await?;

    Ok(())
}

async fn load_persisted_cursor(conn: &mut PgConnection) -> Result<Option<String>, anyhow::Error> {
    let opt_record = sqlx::query_scalar::<_, String>(
        r#"
            SELECT cursor FROM substreams_cursor WHERE name = $1
        "#,
    )
    .bind("ethereum")
    .fetch_optional(conn)
    .await?;

    Ok(opt_record)
}

async fn process_payment_event(
    remittance_events: RemittanceEvents,
    conn: &mut PgConnection,
    chain_id: &ChainId,
    cashier_account_address: &str,
    block_id: String,
) -> Result<(), Error> {
    for payment_event in remittance_events.events {
        // Parse invoice_id from hex string to bytes
        let invoice_id_hex = payment_event.invoice_id.strip_prefix("0x").unwrap_or(&payment_event.invoice_id);
        let invoice_id_bytes = hex::decode(invoice_id_hex)
            .map_err(|e| anyhow!("Failed to decode invoice_id hex: {}", e))?;

        // Convert to fixed-size array
        if invoice_id_bytes.len() != 32 {
            error!("Invalid invoice_id length: expected 32 bytes, got {}", invoice_id_bytes.len());
            continue;
        }
        let mut invoice_id_array = [0u8; 32];
        invoice_id_array.copy_from_slice(&invoice_id_bytes);

        let (is_mint, quote_id, quote_amount, unit) = if let Some((quote_id, amount, unit)) =
            db_node::mint_quote::get_quote_infos_by_invoice_id::<Unit>(
                conn,
                &invoice_id_array,
            )
            .await?
        {
            (true, quote_id, amount, unit)
        } else if let Some((quote_id, amount, unit)) =
            db_node::melt_quote::get_quote_infos_by_invoice_id::<Unit>(
                conn,
                &invoice_id_array,
            )
            .await?
        {
            (false, quote_id, amount, unit)
        } else {
            error!("no quote for invoice_id {}", payment_event.invoice_id);
            continue;
        };

        let on_chain_constants = ON_CHAIN_CONSTANTS
            .get(chain_id.as_str())
            .ok_or(anyhow!("unknown chain id {}", chain_id))?;

        // Parse asset address from hex string
        let asset_address = payment_event.asset.strip_prefix("0x").unwrap_or(&payment_event.asset);
        let asset = match on_chain_constants
            .assets_contract_address
            .get_asset_for_contract_address_hex(asset_address)
        {
            Some(asset) => asset,
            None => {
                error!(
                    r#"Got an event for token with address {} which doesn't match any known asset.
                    This is not supposed to happen as we configure both at compile time."#,
                    payment_event.asset
                );
                continue;
            }
        };

        if !unit.is_asset_supported(asset) {
            debug!(
                "Got payment for quote {}, that expect asset {}, using asset {}, which is not the expected one.",
                quote_id, asset, asset
            );
            continue;
        }

        #[allow(clippy::collapsible_else_if)]
        if is_mint {
            let payee_address = payment_event.payee.strip_prefix("0x").unwrap_or(&payment_event.payee);
            let cashier_address = cashier_account_address.strip_prefix("0x").unwrap_or(cashier_account_address);
            if payee_address.eq_ignore_ascii_case(cashier_address) {
                let db_event = PaymentEvent {
                    block_id: block_id.clone(),
                    tx_hash: payment_event.tx_hash,
                    index: payment_event.log_index as i64,
                    asset: payment_event.asset,
                    payee: payment_event.payee,
                    invoice_id: invoice_id_array,
                    payer: payment_event.payer,
                    amount_low: payment_event.amount.clone(),
                    amount_high: "0x0".to_string(), // Ethereum uses single 256-bit values
                };

                handle_mint_payment(conn, quote_id, db_event, unit, quote_amount).await?;
            }
        } else {
            if let Some(payee_address) = on_chain_constants.payee_address.as_ref() {
                let event_payee = payment_event.payee.strip_prefix("0x").unwrap_or(&payment_event.payee);
                let expected_payee = payee_address.strip_prefix("0x").unwrap_or(payee_address);
                if event_payee.eq_ignore_ascii_case(expected_payee) {
                    let db_event = PaymentEvent {
                        block_id: block_id.clone(),
                        tx_hash: payment_event.tx_hash,
                        index: payment_event.log_index as i64,
                        asset: payment_event.asset,
                        payee: payment_event.payee,
                        invoice_id: invoice_id_array,
                        payer: payment_event.payer,
                        amount_low: payment_event.amount.clone(),
                        amount_high: "0x0".to_string(),
                    };

                    handle_melt_payment(conn, quote_id, db_event, unit, quote_amount).await?;
                }
            }
        }
    }

    Ok(())
}

async fn handle_mint_payment(
    db_conn: &mut PgConnection,
    quote_id: Uuid,
    payment_event: PaymentEvent,
    unit: Unit,
    quote_amount: Amount,
) -> Result<(), Error> {
    db_node::mint_payment_event::insert_new_payment_event(db_conn, &payment_event).await?;

    let current_paid =
        db_node::mint_payment_event::get_current_paid(db_conn, &payment_event.invoice_id)
            .await?
            .map(|(low, _high)| -> Result<primitive_types::U256, Error> {
                // For Ethereum, we only use the low part since it's a single 256-bit value
                let amount_hex = low.strip_prefix("0x").unwrap_or(&low);
                let amount_bytes = hex::decode(amount_hex)?;
                Ok(primitive_types::U256::from_big_endian(&amount_bytes))
            })
            .try_fold(primitive_types::U256::zero(), |acc, a| {
                match a {
                    Ok(v) => v.checked_add(acc).ok_or(anyhow!(
                        "u256 value overflowed during the computation of the total amount paid for invoice"
                    )),
                    Err(e) => Err(e),
                }
            })?;

    let to_pay = unit.convert_amount_into_u256(quote_amount);
    if current_paid >= to_pay {
        db_node::mint_quote::set_state(db_conn, quote_id, MintQuoteState::Paid).await?;
        event!(
            name: "mint-quote-paid",
            Level::INFO,
            name = "mint-quote-paid",
            %quote_id,
        );
    }

    Ok(())
}

async fn handle_melt_payment(
    db_conn: &mut PgConnection,
    quote_id: Uuid,
    payment_event: PaymentEvent,
    unit: Unit,
    quote_amount: Amount,
) -> Result<(), Error> {
    db_node::melt_payment_event::insert_new_payment_event(db_conn, &payment_event).await?;

    let current_paid =
        db_node::melt_payment_event::get_current_paid(db_conn, &payment_event.invoice_id)
            .await?
            .map(|(low, _high)| -> Result<primitive_types::U256, Error> {
                // For Ethereum, we only use the low part since it's a single 256-bit value
                let amount_hex = low.strip_prefix("0x").unwrap_or(&low);
                let amount_bytes = hex::decode(amount_hex)?;
                Ok(primitive_types::U256::from_big_endian(&amount_bytes))
            })
            .try_fold(primitive_types::U256::zero(), |acc, a| {
                match a {
                    Ok(v) => v.checked_add(acc).ok_or(anyhow!(
                        "u256 value overflowed during the computation of the total amount paid for invoice"
                    )),
                    Err(e) => Err(e),
                }
            })?;

    let to_pay = unit.convert_amount_into_u256(quote_amount);
    if current_paid >= to_pay {
        db_node::melt_quote::set_state(db_conn, quote_id, MeltQuoteState::Paid).await?;
        event!(
            name: "melt-quote-paid",
            Level::INFO,
            name = "melt-quote-paid",
            %quote_id,
        );
    }

    Ok(())
}
