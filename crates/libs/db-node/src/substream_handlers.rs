use crate::{PaymentEvent, melt_payment_event, melt_quote, mint_payment_event, mint_quote};
use anyhow::{Error, Result, anyhow};
use nuts::{Amount, nut04::MintQuoteState, nut05::MeltQuoteState, traits::Unit256};
use primitive_types::U256;
use sqlx::{PgConnection, types::Uuid};
use tracing::{Level, event};

pub async fn handle_mint_payment<U: Unit256>(
    db_conn: &mut PgConnection,
    quote_id: Uuid,
    payment_event: PaymentEvent,
    unit: U,
    quote_amount: Amount,
) -> Result<(), Error> {
    mint_payment_event::insert_new_payment_event(db_conn, &payment_event).await?;

    let current_paid = mint_payment_event::get_current_paid(db_conn, &payment_event.invoice_id)
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
        mint_quote::set_state(db_conn, quote_id, MintQuoteState::Paid).await?;
        event!(
            name: "mint-quote-paid",
            Level::INFO,
            name = "mint-quote-paid",
            %quote_id,
        );
    }

    Ok(())
}

pub async fn handle_melt_payment<U: Unit256>(
    db_conn: &mut PgConnection,
    quote_id: Uuid,
    payment_event: PaymentEvent,
    unit: U,
    quote_amount: Amount,
) -> Result<(), Error> {
    melt_payment_event::insert_new_payment_event(db_conn, &payment_event).await?;

    let current_paid = melt_payment_event::get_current_paid(db_conn, &payment_event.invoice_id)
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
        melt_quote::set_state(db_conn, quote_id, MeltQuoteState::Paid).await?;
        event!(
            name: "mint-quote-paid",
            Level::INFO,
            name = "mint-quote-paid",
            %quote_id,
        );
    }

    Ok(())
}
