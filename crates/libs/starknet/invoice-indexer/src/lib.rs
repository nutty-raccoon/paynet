mod abi;
mod pb;

use crate::abi::invoice_contract_contract::Event as Invoice_ContractEvent;
use crate::pb::sf::substreams::starknet::r#type::v1::Transactions;
use pb::starknet::v1::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use starknet::core::types::EmittedEvent;
use starknet::core::types::Felt;
use substreams::Hex;
use substreams::log;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;

#[derive(Deserialize)]
struct Amount {
    low: u128,
    high: u128,
}
#[derive(Deserialize)]
struct Remittance {
    asset: String,
    payee: String,
    invoice_id: String,
    payer: String,
    amount: Amount,
}

#[derive(Deserialize)]
struct EventData {
    #[serde(rename = "Remittance")]
    remittance: Remittance,
}

fn parse_event(event: &Event, block_id: String, tx_hash: String) -> Option<InvoiceContractEvent> {
    let event_data: EventData = serde_json::from_str(&event.json_description).unwrap();
    let remittance = event_data.remittance;

    Some(InvoiceContractEvent {
        block_id,
        tx_hash,
        event_index: 0,
        asset: remittance.asset,
        payee: remittance.payee,
        invoice_id: remittance.invoice_id,
        payer: remittance.payer,
        amount_low: remittance.amount.low.to_string(),
        amount_high: remittance.amount.high.to_string(),
    })
}

#[substreams::handlers::map]
fn map_invoice_contract_events(
    transactions: Transactions,
) -> Result<Events, substreams::errors::Error> {
    let mut parsed_events: Vec<InvoiceContractEvent> = vec![];
    if transactions.transactions_with_receipt.is_empty() {
        log::info!("No transactions with receipts found");
        return Ok(Events::default());
    }

    let block_id = transactions.clock.unwrap().id.clone();
    for transaction in transactions.transactions_with_receipt {
        let data = transaction.receipt.clone().unwrap();

        let data_events = data.events;
        for event in data_events {
            let event_from_address = Hex(event.from_address.as_slice()).to_string();

            if event_from_address
                != "026b2c472aa4ea32fc12f6c44707712552eff4aac48dd75c870e79b8a3fb676e"
            {
                continue;
            }
            log::info!("Processing event from address: {}", event_from_address);

            let mut data_felts = vec![];
            let mut keys_felts = vec![];
            for key in event.keys {
                let key = Felt::from_bytes_be_slice(key.as_slice());
                keys_felts.push(key);
            }

            for bytes in event.data {
                let felt = Felt::from_bytes_be_slice(bytes.as_slice());
                data_felts.push(felt);
            }

            let emitted_event = EmittedEvent {
                from_address: Felt::from_bytes_be_slice(event.from_address.as_slice()),
                keys: keys_felts,
                data: data_felts,
                block_hash: None,
                block_number: None,
                transaction_hash: Felt::default(),
            };
            log::info!(
                "Emitted event: {:?} with keys: {:?} and data: {:?}",
                emitted_event,
                emitted_event.keys,
                emitted_event.data
            );

            let invoice_contract_event = Invoice_ContractEvent::try_from(emitted_event).unwrap();

            let event_json = serde_json::to_string(&invoice_contract_event).unwrap();

            log::info!("Parsed event JSON: {}", event_json);
            let event = Event {
                json_description: event_json,
            };

            let parsed_event = parse_event(
                &event,
                block_id.clone(),
                String::from_utf8_lossy(&transaction.receipt.clone().unwrap().transaction_hash)
                    .to_string(),
            );

            if parsed_event.is_none() {
                log::info!("Failed to parse event: {:?}", event);
                continue;
            }
            if let Some(invoice_event) = parsed_event {
                parsed_events.push(invoice_event);
            }
        }
    }
    log::info!("Parsed {} invoice contract events", parsed_events.len());

    Ok(Events {
        events: parsed_events,
    })
}

#[substreams::handlers::map]
fn db_out(events: Events) -> DatabaseChanges {
    let mut tables: DatabaseChangeTables = DatabaseChangeTables::new();

    // Handle invoice events
    for invoice_event in events.events {
        let mut hasher = Sha256::new();
        hasher.update(&invoice_event.block_id);
        hasher.update(&invoice_event.tx_hash);
        hasher.update(invoice_event.event_index.to_le_bytes());
        hasher.update(&invoice_event.asset);
        hasher.update(&invoice_event.payee);
        hasher.update(&invoice_event.invoice_id);
        hasher.update(&invoice_event.payer);
        hasher.update(&invoice_event.amount_low);
        hasher.update(&invoice_event.amount_high);
        let pk = Hex::encode(hasher.finalize());

        tables
            .create_row("invoice_contract_events", pk)
            .set("block_id", invoice_event.block_id)
            .set("tx_hash", invoice_event.tx_hash)
            .set("event_index", invoice_event.event_index.to_string())
            .set("asset", invoice_event.asset)
            .set("payee", invoice_event.payee)
            .set("invoice_id", invoice_event.invoice_id)
            .set("payer", invoice_event.payer)
            .set("amount_low", invoice_event.amount_low)
            .set("amount_high", invoice_event.amount_high);
    }

    tables.to_database_changes()
}
