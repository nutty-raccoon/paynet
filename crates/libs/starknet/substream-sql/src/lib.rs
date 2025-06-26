mod pb;
use core::panic;
use sha2::{Digest, Sha256};
use substream_paynet::pb::starknet::v1::Events;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;

#[substreams::handlers::map]
fn db_out(input: Events) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = DatabaseChangeTables::new();

    // Skip if no events to process
    if input.events.is_empty() {
        return Ok(tables.to_database_changes());
    }

    // Process each PaymentEvent in the input Vec
    for payment_event in input.events.iter() {
        // Create a unique ID for the payment event using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(payment_event.index.to_le_bytes());
        hasher.update(&payment_event.payee);
        hasher.update(&payment_event.asset);
        hasher.update(&payment_event.invoice_id);
        hasher.update(&payment_event.payer);
        hasher.update(&payment_event.amount_low);
        hasher.update(&payment_event.amount_high);
        let payment_event_id = format!("{:x}", hasher.finalize());

        let payment_row = tables.create_row("payment_event", &payment_event_id);
        payment_row.set("event_index", payment_event.index);
        payment_row.set("payee", &payment_event.payee);
        payment_row.set("asset", &payment_event.asset);
        payment_row.set("invoice_id", &payment_event.invoice_id);
        payment_row.set("payer", &payment_event.payer);
        payment_row.set("amount_low", &payment_event.amount_low);
        payment_row.set("amount_high", &payment_event.amount_high);
    }

    Ok(tables.to_database_changes())
}
