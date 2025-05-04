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
    quote_hash: &[u8; 32],
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
        r#"INSERT INTO melt_quote (id, invoice_id, unit, amount, fee, request, expiry, state) VALUES ($1, $2, $3, $4, $5, $6, $7, 'UNPAID')"#,
        quote_id,
        quote_hash,
        &unit.to_string(),
        amount.into_i64_repr(),
        fee.into_i64_repr(),
        request,
        expiry,
    ).execute(conn).await?;

    Ok(())
}

pub async fn build_response_from_db(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MeltQuoteResponse<Uuid>, Error> {
    let record = sqlx::query!(
        r#"SELECT amount, fee, state AS "state: MeltQuoteState", expiry, transfer_id AS "transfer_id!" FROM melt_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let expiry = record
        .expiry
        .unix_timestamp()
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;
    let amount = Amount::from_i64_repr(record.amount);
    let fee = Amount::from_i64_repr(record.fee);

    Ok(MeltQuoteResponse {
        quote: quote_id,
        amount,
        fee,
        state: record.state,
        expiry,
        transfer_id: record.transfer_id,
    })
}

pub async fn get_data<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<(U, Amount, Amount, MeltQuoteState, u64), Error> {
    let record = sqlx::query!(
        r#"SELECT unit, amount, fee, state AS "state: MeltQuoteState", expiry FROM melt_quote where id = $1"#,
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

    Ok((unit, amount, fee, record.state, expiry))
}

pub async fn get_state(conn: &mut PgConnection, quote_id: Uuid) -> Result<MeltQuoteState, Error> {
    let record = sqlx::query!(
        r#"SELECT state AS "state: MeltQuoteState" FROM melt_quote where id = $1"#,
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

pub async fn register_transfer_id(
    conn: &mut PgConnection,
    quote_id: Uuid,
    transfer_id: &[u8],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE melt_quote
            SET transfer_id = $2
            WHERE id = $1
        "#,
        quote_id,
        transfer_id,
    )
    .execute(conn)
    .await?;

    Ok(())
}
