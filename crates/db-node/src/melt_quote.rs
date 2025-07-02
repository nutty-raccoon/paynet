use nuts::{
    Amount,
    nut05::{MeltQuoteResponse, MeltQuoteState},
    traits::Unit,
};
use sqlx::{PgConnection, types::time::OffsetDateTime};
use uuid::Uuid;

use crate::Error;

// TODO: use a struct and ToSql trait instead
#[allow(clippy::too_many_arguments)]
pub async fn insert_new<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
    invoice_id: &[u8; 32],
    unit: U,
    amount: Amount,
    fee: Amount,
    request: &str,
    expiry: u64,
) -> Result<(), Error> {
    let expiry: i64 = expiry
        .try_into()
        .map_err(|_| Error::RuntimeToDbConversion)?;
    let expiry =
        OffsetDateTime::from_unix_timestamp(expiry).map_err(|_| Error::RuntimeToDbConversion)?;

    sqlx::query!(
        r#"
        INSERT INTO melt_quote
            (id, invoice_id, unit, amount, fee, request, expiry, state)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, 'UNPAID')"#,
        quote_id,
        invoice_id,
        &unit.to_string(),
        amount.into_i64_repr(),
        fee.into_i64_repr(),
        request,
        expiry,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn build_response_from_db<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MeltQuoteResponse<Uuid, U>, Error> {
    let record = sqlx::query!(
        r#"
        SELECT 
            amount, 
            unit,
            state AS "state: MeltQuoteState", 
            expiry
        FROM melt_quote
        WHERE id = $1
        "#,
        quote_id
    )
    .fetch_one(conn)
    .await;

    let record = record?;
    let expiry = record
        .expiry
        .unix_timestamp()
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;
    let amount = Amount::from_i64_repr(record.amount);
    let unit = U::from_str(&record.unit).map_err(|_| Error::DbToRuntimeConversion)?;

    Ok(MeltQuoteResponse {
        quote: quote_id,
        unit,
        amount,
        state: record.state,
        expiry,
    })
}

pub async fn get_data<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<(U, Amount, Amount, MeltQuoteState, u64, [u8; 32], String), Error> {
    let record = sqlx::query!(
        r#"SELECT unit, amount, fee, state AS "state: MeltQuoteState", invoice_id, expiry, request FROM melt_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let unit = U::from_str(&record.unit).map_err(|_| Error::DbToRuntimeConversion)?;
    let amount = Amount::from_i64_repr(record.amount);
    let fee = Amount::from_i64_repr(record.fee);
    let expiry = record
        .expiry
        .unix_timestamp()
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;

    let quote_hash: [u8; 32] = record
        .invoice_id
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;

    Ok((
        unit,
        amount,
        fee,
        record.state,
        expiry,
        quote_hash,
        record.request,
    ))
}

pub async fn get_state(conn: &mut PgConnection, quote_id: Uuid) -> Result<MeltQuoteState, Error> {
    let record = sqlx::query!(
        r#"SELECT state AS "state: MeltQuoteState" FROM melt_quote WHERE id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    Ok(record.state)
}

pub async fn set_state(
    conn: &mut PgConnection,
    quote_id: Uuid,
    state: MeltQuoteState,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE melt_quote
            SET state = $2
            WHERE id = $1
        "#,
        quote_id,
        state as MeltQuoteState,
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn get_quote_infos_by_invoice_id<U: Unit>(
    conn: &mut PgConnection,
    invoice_id: &[u8; 32],
) -> Result<Option<(Uuid, Amount, U)>, Error> {
    let record = sqlx::query!(
        r#"
            SELECT id, amount, unit from melt_quote WHERE invoice_id = $1 LIMIT 1
        "#,
        invoice_id
    )
    .fetch_optional(conn)
    .await?;

    let ret = if let Some(record) = record {
        let quote_id = record.id;
        let amount = Amount::from_i64_repr(record.amount);
        let unit = U::from_str(&record.unit).map_err(|_| Error::DbToRuntimeConversion)?;
        Some((quote_id, amount, unit))
    } else {
        None
    };

    Ok(ret)
}

pub async fn get_state_and_transfer_ids(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<(MeltQuoteState, Option<Vec<String>>), Error> {
    let record = sqlx::query!(
        r#"SELECT
            mq.state AS "state: MeltQuoteState",
            COALESCE(ARRAY_AGG(mpe.tx_hash) FILTER (WHERE mpe.tx_hash IS NOT NULL), '{}') AS "tx_hashes"
        FROM melt_quote mq LEFT JOIN melt_payment_event mpe ON mq.invoice_id = mpe.invoice_id
        WHERE mq.id = $1
        GROUP BY mq.state"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    Ok((record.state, record.tx_hashes))
}
