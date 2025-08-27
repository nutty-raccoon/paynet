use anyhow::{Error, Result, anyhow, format_err};
use db_node::PaymentEvent;
use eth_types::{ChainId, Unit, constants::ON_CHAIN_CONSTANTS};
use ethers::types::Address;
use futures::StreamExt;
use http::Uri;
use nuts::{Amount, nut04::MintQuoteState, nut05::MeltQuoteState};
use primitive_types::U256;
use prost::Message;
use sqlx::{
    PgConnection, PgPool,
    types::{
        Uuid,
        chrono::{DateTime, Utc},
    },
};
use std::{
    env::{self, VarError},
    sync::Arc,
};
use substreams_streams::parse_inputs;
use substreams_streams::pb::{
    eth_invoice_contract::v1::RemittanceEvents, sf::substreams::rpc::v2::BlockScopedData,
};
use substreams_streams::stream::{BlockResponse, SubstreamsStream};
use substreams_streams::substreams::SubstreamsEndpoint;
use tracing::{Level, debug, error, event};

pub async fn launch(
    pg_pool: PgPool,
    endpoint_url: Uri,
    chain_id: ChainId,
    cashier_account_address: Address,
) -> Result<()> {
    let package_path = "./eth-invoice-susbtream-v0.1.6.spkg";
    let package = parse_inputs::read_package(package_path, vec![])?;

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
    cashier_account_address: Address,
) -> Result<(), Error> {
    let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

    let clock = data.clock.as_ref().unwrap();
    let timestamp = clock.timestamp.as_ref().unwrap();
    let date = DateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
        .expect("received timestamp should always be valid");

    sqlx::query(r#"
            INSERT INTO substreams_eth_block (id, number, timestamp) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING;
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
    sqlx::query!(
        r#"
            DELETE FROM substreams_eth_block WHERE number > $1;
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
        "ethereum",
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
        "ethereum"
    )
    .fetch_optional(conn)
    .await?;

    Ok(opt_record.map(|r| r.cursor))
}

async fn process_payment_event(
    remittance_events: RemittanceEvents,
    conn: &mut PgConnection,
    chain_id: &ChainId,
    cashier_account_address: Address,
    block_id: String,
) -> Result<(), Error> {
    for payment_event in remittance_events.events {
        let invoice_id: [u8; 32] = payment_event.invoice_id.as_slice().try_into()?;
        let (is_mint, quote_id, quote_amount, unit) = if let Some((quote_id, amount, unit)) =
            db_node::mint_quote::get_quote_infos_by_invoice_id::<Unit>(conn, &invoice_id).await?
        {
            (true, quote_id, amount, unit)
        } else if let Some((quote_id, amount, unit)) =
            db_node::melt_quote::get_quote_infos_by_invoice_id::<Unit>(conn, &invoice_id).await?
        {
            (false, quote_id, amount, unit)
        } else {
            error!("no quote for invoice_id  0x{}", hex::encode(invoice_id));
            continue;
        };

        let on_chain_constants = ON_CHAIN_CONSTANTS
            .get(chain_id.as_str())
            .ok_or(anyhow!("unkonwn chain id {}", chain_id))?;

        let asset = Address::from_slice(&payment_event.asset);
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

        let amount_u256 = U256::from_dec_str(&payment_event.amount).unwrap();

        #[allow(clippy::collapsible_else_if)]
        if is_mint {
            let payee = Address::from_slice(&payment_event.payee);
            if payee == cashier_account_address {
                let db_event = PaymentEvent {
                    block_id: block_id.clone(),
                    tx_hash: hex::encode(&payment_event.tx_hash),
                    index: i64::from(payment_event.event_index),
                    asset: hex::encode(&payment_event.asset),
                    payee: hex::encode(&payment_event.payee),
                    invoice_id,
                    payer: hex::encode(&payment_event.payer),
                    amount_low: hex::encode(amount_u256.to_little_endian()),
                    amount_high: hex::encode(amount_u256.to_big_endian()),
                };
                handle_mint_payment(conn, quote_id, db_event, unit, quote_amount).await?;
            }
        } else {
            let payer = Address::from_slice(&payment_event.payer);
            if payer == cashier_account_address {
                let db_event = PaymentEvent {
                    block_id: block_id.clone(),
                    tx_hash: hex::encode(&payment_event.tx_hash),
                    index: i64::from(payment_event.event_index),
                    asset: hex::encode(&payment_event.asset),
                    payee: hex::encode(&payment_event.payee),
                    invoice_id,
                    payer: hex::encode(&payment_event.payer),
                    amount_low: hex::encode(amount_u256.to_little_endian()),
                    amount_high: hex::encode(amount_u256.to_big_endian()),
                };
                handle_melt_payment(conn, quote_id, db_event, unit, quote_amount).await?;
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
            .map(|(low_hex, high_hex)| -> Result<U256, Error> {
                let low_bytes = hex::decode(low_hex)?;
                let high_bytes = hex::decode(high_hex)?;

                let from_low = U256::from_little_endian(&low_bytes);
                let from_high = U256::from_big_endian(&high_bytes);

                if from_low != from_high {
                    return Err(anyhow!(
                        "Mismatch between low-endian and high-endian decoded U256 values"
                    ));
                }

                Ok(from_low)
            })
            .try_fold(U256::zero(), |acc, res| match res {
                Ok(val) => val
                    .checked_add(acc)
                    .ok_or_else(|| anyhow!("U256 overflow while summing total paid amount")),
                Err(err) => Err(err),
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
    db_node::mint_payment_event::insert_new_payment_event(db_conn, &payment_event).await?;

    let current_paid =
        db_node::mint_payment_event::get_current_paid(db_conn, &payment_event.invoice_id)
            .await?
            .map(|(low_hex, high_hex)| -> Result<U256, Error> {
                let low_bytes = hex::decode(low_hex)?;
                let high_bytes = hex::decode(high_hex)?;

                let from_low = U256::from_little_endian(&low_bytes);
                let from_high = U256::from_big_endian(&high_bytes);

                if from_low != from_high {
                    return Err(anyhow!(
                        "Mismatch between low-endian and high-endian decoded U256 values"
                    ));
                }

                Ok(from_low)
            })
            .try_fold(U256::zero(), |acc, res| match res {
                Ok(val) => val
                    .checked_add(acc)
                    .ok_or_else(|| anyhow!("U256 overflow while summing total paid amount")),
                Err(err) => Err(err),
            })?;

    let to_pay = unit.convert_amount_into_u256(quote_amount);
    if current_paid >= to_pay {
        db_node::melt_quote::set_state(db_conn, quote_id, MeltQuoteState::Paid).await?;
        event!(
            name: "mint-quote-paid",
            Level::INFO,
            name = "mint-quote-paid",
            %quote_id,
        );
    }

    Ok(())
}
