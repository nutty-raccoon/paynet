use sqlx::{PgConnection, types::time::OffsetDateTime};

pub async fn insert_new_payment_event(
    db_conn: &mut PgConnection,
    payment_event: &PaymentEvent,
) -> Result<(), Error> {

    const INSERT_PAYMENT_EVENT: &str = r#"
        INSERT INTO payment_event (
            block_id, 
            tx_hash, 
            event_index, 
            asset, 
            invoice_id, 
            amount_low, 
            amount_high
            )
        VALUES
            ($1, $2, $3, $4, $5, $6, $7)
        )
    "#;

    db_conn.execute(
        INSERT_PAYMENT_EVENT,
        (
            &payment_event.block_id,
            &payment_event.tx_hash,
            &payment_event.asset,
            &payment_event.invoice_id,
            &payment_event.amount_low,
            &payment_event.amount_high
        ),
    )?;
    
    Ok(())
}