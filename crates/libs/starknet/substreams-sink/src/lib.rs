use std::{
    env::{self, VarError},
    str::FromStr,
    sync::Arc,
};

use crate::pb::{invoice_contract::v1::RemittanceEvents, sf::substreams::rpc::v2::BlockScopedData};
use anyhow::{Error, Result, anyhow};
use db_node::PaymentEvent;
use futures::StreamExt;
use http::Uri;
use nuts::{Amount, nut04::MintQuoteState, nut05::MeltQuoteState};
use pb::{
    invoice_contract::v1::RemittanceEvent,
    sf::substreams::v1::module::input::{Input, Params},
};
use prost::Message;
use sqlx::{
    PgConnection, PgPool,
    types::{
        Uuid,
        chrono::{DateTime, Utc},
    },
};
use starknet::core::types::Felt;
use starknet_types::{ChainId, StarknetU256, Unit, constants::ON_CHAIN_CONSTANTS};
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
    initial_block: i64,
    cashier_account_address: Felt,
) -> Result<()> {
    const OUTPUT_MODULE_NAME: &str = "map_invoice_contract_events";
    const STARKNET_FILTERED_TRANSACTIONS_MODULE_NAME: &str = "starknet:filtered_transactions";

    let mut package = parse_inputs::read_package(vec![])?;

    let token = match env::var("SUBSTREAMS_API_TOKEN") {
        Err(VarError::NotPresent) => None,
        Err(e) => Err(e)?,
        Ok(val) if val.is_empty() => None,
        Ok(val) => Some(val),
    };

    let on_chain_constants = ON_CHAIN_CONSTANTS
        .get(chain_id.as_str())
        .ok_or(anyhow!("unsuported chain id"))?;

    let starknet_filtered_transactions_expression = format!(
        "ev:from_address:{}",
        on_chain_constants
            .invoice_payment_contract_address
            .to_fixed_hex_string()
    );
    // Update tx filter
    package
        .modules
        .as_mut()
        .unwrap()
        .modules
        .iter_mut()
        .find(|m| m.name == STARKNET_FILTERED_TRANSACTIONS_MODULE_NAME)
        .ok_or(anyhow!(
            "module `{}` not found",
            STARKNET_FILTERED_TRANSACTIONS_MODULE_NAME
        ))?
        .inputs[0]
        .input = Some(Input::Params(Params {
        value: starknet_filtered_transactions_expression,
    }));

    let endpoint = Arc::new(SubstreamsEndpoint::new(endpoint_url, token).await?);

    let mut db_conn = pg_pool.acquire().await?;

    let cursor: Option<String> = load_persisted_cursor(&mut db_conn).await?;

    let mut stream = SubstreamsStream::new(
        endpoint,
        cursor,
        package.modules,
        OUTPUT_MODULE_NAME.to_string(),
        initial_block,
        0,
    );

    loop {
        match stream.next().await {
            None => {
                break;
            }
            Some(Ok(BlockResponse::New(data))) => {
                process_block_scoped_data(&mut db_conn, &data, &chain_id, cashier_account_address)
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
    cashier_account_address: Felt,
) -> Result<(), Error> {
    let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

    let clock = data.clock.as_ref().unwrap();
    let timestamp = clock.timestamp.as_ref().unwrap();
    let date = DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
        .expect("received timestamp should always be valid");

    let events = RemittanceEvents::decode(output.value.as_slice())?;

    println!(
        "Block #{} - Payload {} ({} bytes) - Drift {}s",
        clock.number,
        output.type_url.replace("type.googleapis.com/", ""),
        output.value.len(),
        -date.signed_duration_since(Utc::now()).num_seconds()
    );

    if !events.events.is_empty() {
        sqlx::query(r#"
            INSERT INTO substreams_starknet_block (id, number, timestamp) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;
        "#)
        .bind(&clock.id)
            .bind(i64::try_from(clock.number).unwrap())
                .bind(date)
        .execute(&mut *conn).await?;

        process_payment_event(
            events.events,
            conn,
            chain_id,
            cashier_account_address,
            clock.id.clone(),
        )
        .await?;
    }

    Ok(())
}

async fn delete_invalid_blocks(
    conn: &mut PgConnection,
    last_valid_block_number: u64,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
            DELETE FROM substreams_starknet_block WHERE number > $1;
        "#,
        i64::try_from(last_valid_block_number).unwrap()
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn persist_cursor(conn: &mut PgConnection, cursor: String) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
            INSERT INTO substreams_cursor (name, cursor) VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET cursor = excluded.cursor
        "#,
        "starknet",
        cursor
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn load_persisted_cursor(conn: &mut PgConnection) -> Result<Option<String>, anyhow::Error> {
    let opt_record = sqlx::query!(
        r#"
            SELECT cursor FROM substreams_cursor WHERE name = $1
        "#,
        "starknet"
    )
    .fetch_optional(conn)
    .await?;

    Ok(opt_record.map(|r| r.cursor))
}

async fn process_payment_event(
    remittance_events: Vec<RemittanceEvent>,
    conn: &mut PgConnection,
    chain_id: &ChainId,
    cashier_account_address: Felt,
    block_id: String,
) -> Result<(), Error> {
    for payment_event in remittance_events {
        let invoice_id = Felt::from_bytes_be_slice(&payment_event.invoice_id);
        let (is_mint, quote_id, quote_amount, unit) = if let Some((quote_id, amount, unit)) =
            db_node::mint_quote::get_quote_infos_by_invoice_id::<Unit>(
                conn,
                &invoice_id.to_bytes_be(),
            )
            .await?
        {
            (true, quote_id, amount, unit)
        } else if let Some((quote_id, amount, unit)) =
            db_node::melt_quote::get_quote_infos_by_invoice_id::<Unit>(
                conn,
                &invoice_id.to_bytes_be(),
            )
            .await?
        {
            (false, quote_id, amount, unit)
        } else {
            error!("no quote for invoice_id {:#x}", invoice_id);
            continue;
        };

        let on_chain_constants = ON_CHAIN_CONSTANTS
            .get(chain_id.as_str())
            .ok_or(anyhow!("unkonwn chain id {}", chain_id))?;

        let asset = Felt::from_bytes_be_slice(&payment_event.asset);
        let asset = match on_chain_constants
            .assets_contract_address
            .get_asset_for_contract_address(asset)
        {
            Some(asset) => asset,
            None => {
                error!(
                    r#"Got an event for token with address {} which doesn't match any known asset.
                    This is not supposed to happen as we configure both at compile time."#,
                    asset
                );
                continue;
            }
        };
        if !unit.is_asset_supported(asset) {
            // Payment was done using an asset that doesn't match the requested unit
            // Could just be someone reusing an already existing invoice id he saw onchain.
            // But it could also be an error in the wallet.
            debug!(
                "Got payment for quote {}, that expect asset {}, using asset {}, which is not the expected one.",
                quote_id, asset, asset
            );
            continue;
        }

        #[allow(clippy::collapsible_else_if)]
        if is_mint {
            let payee = Felt::from_bytes_be_slice(&payment_event.payee);
            if payee == cashier_account_address {
                let db_event = PaymentEvent {
                    block_id: block_id.clone(),
                    tx_hash: Felt::from_bytes_be_slice(&payment_event.tx_hash).to_hex_string(),
                    index: i64::try_from(payment_event.event_index).unwrap(),
                    asset: Felt::from_bytes_be_slice(&payment_event.asset).to_hex_string(),
                    payee: Felt::from_bytes_be_slice(&payment_event.payee).to_hex_string(),
                    invoice_id: Felt::from_bytes_be_slice(&payment_event.invoice_id).to_bytes_be(),
                    payer: Felt::from_bytes_be_slice(&payment_event.payer).to_hex_string(),
                    amount_low: Felt::from_bytes_be_slice(&payment_event.amount_low)
                        .to_hex_string(),
                    amount_high: Felt::from_bytes_be_slice(&payment_event.amount_high)
                        .to_hex_string(),
                };
                handle_mint_payment(conn, quote_id, db_event, unit, quote_amount).await?;
            }
        } else {
            let payer = Felt::from_bytes_be_slice(&payment_event.payer);
            if payer == cashier_account_address {
                let db_event = PaymentEvent {
                    block_id: block_id.clone(),
                    tx_hash: Felt::from_bytes_be_slice(&payment_event.tx_hash).to_hex_string(),
                    index: i64::try_from(payment_event.event_index).unwrap(),
                    asset: Felt::from_bytes_be_slice(&payment_event.asset).to_hex_string(),
                    payee: Felt::from_bytes_be_slice(&payment_event.payee).to_hex_string(),
                    invoice_id: Felt::from_bytes_be_slice(&payment_event.invoice_id).to_bytes_be(),
                    payer: Felt::from_bytes_be_slice(&payment_event.payer).to_hex_string(),
                    amount_low: Felt::from_bytes_be_slice(&payment_event.amount_low)
                        .to_hex_string(),
                    amount_high: Felt::from_bytes_be_slice(&payment_event.amount_high)
                        .to_hex_string(),
                };
                handle_melt_payment(conn, quote_id, db_event, unit, quote_amount).await?;
            }
        }
    }

    Ok(())
}

// Yeah I know it's basically the same code copied and pasted.
// For now it's fine, better this than adding trait and struct and so on.
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
            .map(|(low, high)| -> Result<primitive_types::U256, Error> {
                let amount_as_strk_256 = StarknetU256 {
                    low: Felt::from_str(&low)?,
                    high: Felt::from_str(&high)?,
                };

                Ok(primitive_types::U256::from(amount_as_strk_256))
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
            .map(|(low, high)| -> Result<primitive_types::U256, Error> {
                let amount_as_strk_256 = StarknetU256 {
                    low: Felt::from_str(&low)?,
                    high: Felt::from_str(&high)?,
                };

                Ok(primitive_types::U256::from(amount_as_strk_256))
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
