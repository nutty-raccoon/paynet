use crate::Error;
use sqlx::PgConnection;
use starknet_payment_indexer::PaymentEvent;

pub async fn insert_new_payment_event(
    db_conn: &mut PgConnection,
    payment_event: &PaymentEvent,
) -> Result<(), Error> {
    sqlx::query!(
        r#" INSERT INTO payment_event (block_id, tx_hash, event_index, asset, invoice_id, amount_low, amount_high) VALUES ($1, $2, $3, $4, $5, $6, $7) "#,
        &payment_event.block_id,
        &payment_event.tx_hash.to_string(),
        i64::from_be_bytes(payment_event.event_idx.to_be_bytes()),
        &payment_event.asset.to_string(),
        &payment_event.invoice_id.to_bytes_be(),
        &payment_event.amount.low.to_string(),
        &payment_event.amount.high.to_string()
    )
    .execute(db_conn)
    .await?;

    Ok(())
}

pub async fn get_current_paid(
    db_conn: &mut PgConnection,
    invoice_id: &[u8; 32],
) -> Result<impl Iterator<Item = (String, String)>, Error> {
    let record = sqlx::query!(
        r#"SELECT  amount_low, amount_high
        FROM payment_event
        WHERE invoice_id = $1"#,
        invoice_id
    )
    .fetch_all(&mut *db_conn)
    .await?;

    let amounts_iterator = record.into_iter().map(|r| (r.amount_low, r.amount_high));

    Ok(amounts_iterator)
}
