#[allow(clippy::enum_variant_names)]
mod pb;

use crate::pb::invoice_contract::v1::{RemittanceEvent, RemittanceEvents};
use crate::pb::sf::substreams::starknet::r#type::v1::Transactions;

// starkli selector Remittance
// 0x027a12f554d018764f982295090da45b4ff0734785be0982b62c329b9ac38033
const REMITTANCE_EVENT_SELECTOR: [u8; 32] = [
    2, 122, 18, 245, 84, 208, 24, 118, 79, 152, 34, 149, 9, 13, 164, 91, 79, 240, 115, 71, 133,
    190, 9, 130, 182, 44, 50, 155, 154, 195, 128, 51,
];

#[substreams::handlers::map]
fn map_invoice_contract_events(
    transactions: Transactions,
) -> Result<RemittanceEvents, substreams::errors::Error> {
    if transactions.transactions_with_receipt.is_empty() {
        return Ok(RemittanceEvents::default());
    }
    let mut remittance_events = Vec::new();
    for transaction in transactions.transactions_with_receipt {
        let receipt = transaction.receipt.unwrap();

        for (index, event) in receipt.events.into_iter().enumerate() {
            let mut keys_iter = event.keys.into_iter();
            // Safe to unwrap as all starknet event have at least one key; the selector.
            let event_selector = keys_iter.next().unwrap();
            if event_selector != REMITTANCE_EVENT_SELECTOR {
                continue;
            }

            // Both safe to unwrap as the `Remittance` event has two extra keys.
            // If this comes to change it is ok to crash
            let asset = keys_iter.next().unwrap();
            let payer = keys_iter.next().unwrap();
            let payee = keys_iter.next().unwrap();

            let mut data_iter = event.data.into_iter();
            let invoice_id = data_iter.next().unwrap();
            let amount_low = data_iter.next().unwrap();
            let amount_high = data_iter.next().unwrap();

            remittance_events.push(RemittanceEvent {
                tx_hash: receipt.transaction_hash.clone(),
                event_index: index.try_into().unwrap(),
                asset,
                payer,
                payee,
                invoice_id,
                amount_low,
                amount_high,
            });
        }
    }

    Ok(RemittanceEvents {
        events: remittance_events,
    })
}
