use nuts::{
    Amount,
    nut04::{MintQuoteResponse, MintQuoteState},
    traits::Unit,
};
use sqlx::{PgConnection, types::time::OffsetDateTime};
use uuid::Uuid;

use crate::Error;

pub async fn insert_new<U: Unit>(
    conn: &mut PgConnection,
    quote_id: Uuid,
    invoice_id: [u8; 32],
    unit: U,
    amount: Amount,
    request: &str,
    expiry: u64,
) -> Result<(), Error> {
    let expiry: i64 = expiry
        .try_into()
        .map_err(|_| Error::RuntimeToDbConversion)?;
    let expiry =
        OffsetDateTime::from_unix_timestamp(expiry).map_err(|_| Error::RuntimeToDbConversion)?;
    sqlx::query!(
        r#"INSERT INTO mint_quote (id, invoice_id, unit, amount, request, expiry, state) VALUES ($1, $2, $3, $4, $5, $6, 'UNPAID')"#,
        quote_id,
        &invoice_id,
        &unit.to_string(),
        amount.into_i64_repr(),
        request,
        expiry,
    ).execute(conn).await?;

    Ok(())
}

pub async fn build_response_from_db(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MintQuoteResponse<Uuid>, Error> {
    let record = sqlx::query!(
        r#"SELECT request, state AS "state: MintQuoteState", expiry FROM mint_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let expiry = record
        .expiry
        .unix_timestamp()
        .try_into()
        .map_err(|_| Error::DbToRuntimeConversion)?;

    Ok(MintQuoteResponse {
        quote: quote_id,
        request: record.request,
        state: record.state,
        expiry,
    })
}

pub async fn get_amount_and_state(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<(Amount, MintQuoteState), Error> {
    let record = sqlx::query!(
        r#"SELECT amount, state AS "state: MintQuoteState" FROM mint_quote where id = $1"#,
        quote_id
    )
    .fetch_one(conn)
    .await?;

    let amount = Amount::from_i64_repr(record.amount);

    Ok((amount, record.state))
}

pub async fn set_state(
    conn: &mut PgConnection,
    quote_id: Uuid,
    state: MintQuoteState,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            UPDATE mint_quote
            SET state = $2
            WHERE id = $1
        "#,
        quote_id,
        state as MintQuoteState
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
            SELECT id, amount, unit from mint_quote WHERE invoice_id = $1 LIMIT 1
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
