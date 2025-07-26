#[allow(clippy::enum_variant_names)]
mod pb;

use crate::pb::invoice_contract::v1::{RemittanceEvent, RemittanceEvents};
use crate::pb::sf::ethereum::r#type::v2::Log;

// Remittance event signature: Remittance(address,address,bytes32,address,uint256)
// keccak256("Remittance(address,address,bytes32,address,uint256)") = 0x...
const REMITTANCE_EVENT_SIGNATURE: [u8; 32] = [
    // This would be the actual keccak256 hash of the event signature
    // For now using placeholder bytes
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34,
    0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
    0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
    0xde, 0xf0,
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

    Some(RemittanceEvent {
        tx_hash: format!("0x{}", hex::encode(&log.receipt.transaction.hash)),
        log_index: log.index,
        asset,
        payee,
        invoice_id,
        payer,
        amount,
    })
}
