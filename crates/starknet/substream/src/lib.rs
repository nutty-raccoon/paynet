mod abi;
mod pb;

use crate::abi::invoice_contract::Event as InvoiceEvent;
use pb::starknet::v1::*;

use crate::pb::sf::substreams::starknet::r#type::v1::Transactions;
use cainome::cairo_serde::CairoSerde;
use num_traits::cast::ToPrimitive;
use starknet::core::types::EmittedEvent;
use starknet::core::types::Felt;
use substreams::log;
use substreams::Hex;

#[substreams::handlers::map]
fn map_invoice_events(transactions: Transactions) -> Result<Events, substreams::errors::Error> {
    let mut proto_events = Events::default();

    for transaction in transactions.transactions_with_receipt {
        let data = transaction.receipt.unwrap();
        let data_events = data.events;

        for (index, event) in data_events.iter().enumerate() {
            let event_from_address = Hex(event.from_address.as_slice()).to_string();

            if event_from_address
                != "076bc499fc0e3f3559cb0a8506e53f313b38a9009dac4356248ea8ae1e5f21ad"
            {
                continue;
            }

            if event.keys.len() < 3 || event.data.len() < 4 {
                log::info!("Skipping event with insufficient keys or data");
                continue;
            }

            let payment_event = PaymentEvent {
                index: index as u64,
                payee: Hex(event.keys[1].as_slice()).to_string(),
                asset: Hex(event.keys[2].as_slice()).to_string(),
                invoice_id: Hex(event.data[0].as_slice()).to_string(),
                payer: Hex(event.data[1].as_slice()).to_string(),
                amount_low: Hex(event.data[2].as_slice()).to_string(),
                amount_high: Hex(event.data[3].as_slice()).to_string(),
            };

            proto_events.events.push(payment_event);
        }
    }

    Ok(proto_events)
}
