#[allow(clippy::enum_variant_names)]
mod pb;

use crate::pb::invoice_contract::v1::{RemittanceEvent, RemittanceEvents};
use crate::pb::sf::ethereum::r#type::v2::Log;

// Remittance event signature: Remittance(address,address,bytes32,address,uint256)
// keccak256("Remittance(address,address,bytes32,address,uint256)") = 0xf67804c4291a6e963fa4351308ed292a548b89d76c77c794c59167f4c6caf108
const REMITTANCE_EVENT_SIGNATURE: [u8; 32] = [
    0xf6, 0x78, 0x04, 0xc4, 0x29, 0x1a, 0x6e, 0x96, 0x3f, 0xa4, 0x35, 0x13, 0x08, 0xed, 0x29, 0x2a,
    0x54, 0x8b, 0x89, 0xd7, 0x6c, 0x77, 0xc7, 0x94, 0xc5, 0x91, 0x67, 0xf4, 0xc6, 0xca, 0xf1, 0x08,
];

#[substreams::handlers::map]
fn map_invoice_contract_events(
    logs: pb::sf::ethereum::r#type::v2::Logs,
) -> Result<RemittanceEvents, substreams::errors::Error> {
    if logs.logs.is_empty() {
        return Ok(RemittanceEvents::default());
    }

    let mut remittance_events = Vec::new();
    
    for log in logs.logs {
        if log.topics.is_empty() {
            continue;
        }

        // Check if this is a Remittance event
        let event_signature = &log.topics[0];
        if event_signature.len() != 32 || event_signature != &REMITTANCE_EVENT_SIGNATURE {
            continue;
        }

        // Parse Remittance event
        if let Some(remittance_event) = parse_remittance_event(&log) {
            remittance_events.push(remittance_event);
        }
    }

    Ok(RemittanceEvents {
        events: remittance_events,
    })
}

fn parse_remittance_event(log: &Log) -> Option<RemittanceEvent> {
    // Remittance event structure:
    // event Remittance(
    //     address indexed asset,
    //     address indexed payee, 
    //     bytes32 indexed invoiceId,
    //     address payer,
    //     uint256 amount
    // );

    if log.topics.len() < 4 {
        return None;
    }

    // Extract indexed parameters from topics
    let asset = format!("0x{}", hex::encode(&log.topics[1][12..32])); // Last 20 bytes for address
    let payee = format!("0x{}", hex::encode(&log.topics[2][12..32])); // Last 20 bytes for address
    let invoice_id = format!("0x{}", hex::encode(&log.topics[3])); // Full 32 bytes for bytes32

    // Extract non-indexed parameters from data
    if log.data.len() < 64 {
        return None;
    }

    let payer = format!("0x{}", hex::encode(&log.data[12..32])); // First 32 bytes, last 20 for address
    let amount = format!("0x{}", hex::encode(&log.data[32..64])); // Second 32 bytes for uint256

    // Get transaction hash from the log's transaction receipt
    let tx_hash = if let Some(receipt) = &log.receipt {
        if let Some(transaction) = &receipt.transaction {
            format!("0x{}", hex::encode(&transaction.hash))
        } else {
            "0x".to_string()
        }
    } else {
        "0x".to_string() // fallback if no receipt
    };

    Some(RemittanceEvent {
        tx_hash,
        log_index: log.index,
        asset,
        payee,
        invoice_id,
        payer,
        amount,
    })
}
