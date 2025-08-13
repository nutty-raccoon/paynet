mod abi;
mod pb;
use pb::invoice_contract::v1::{RemittanceEvent, RemittanceEvents};
use substreams::errors::Error;
use substreams::Hex;
use substreams_ethereum::pb::eth::v2 as eth;
use substreams_ethereum::Event;

#[allow(unused_imports)]
use num_traits::cast::ToPrimitive;
#[allow(unused_imports)]
use std::str::FromStr;
#[allow(unused_imports)]
use substreams::scalar::BigDecimal;

substreams_ethereum::init!();

#[substreams::handlers::map]
fn map_invoice_contract_events(
    contract_address: String,
    blk: eth::Block,
) -> Result<RemittanceEvents, substreams::errors::Error> {
    let decoded_address = verify_parameter(&contract_address)?;

    let mut remittance_events = Vec::new();

    remittance_events.append(
        &mut blk
            .receipts()
            .flat_map(|view| {
                view.receipt
                    .logs
                    .iter()
                    .filter(|log| log.address == decoded_address)
                    .filter_map(|log| {
                        if let Some(event) =
                            abi::invoice_contract::events::Remittance::match_and_decode(log)
                        {
                            return Some(RemittanceEvent {
                                tx_hash: Hex(&view.transaction.hash).to_string(),
                                event_index: log.block_index,
                                amount: event.amount.to_string(),
                                asset: event.asset,
                                invoice_id: event.invoice_id.to_vec(),
                                payee: event.payee,
                                payer: event.payer,
                            });
                        }
                        None
                    })
            })
            .collect(),
    );

    Ok(RemittanceEvents {
        events: remittance_events,
    })
}

pub fn verify_parameter(address: &str) -> Result<Vec<u8>, Error> {
    let normalized = address.strip_prefix("0x").unwrap_or(address);
    if normalized.len() != 40 {
        return Err(Error::msg("invalid Ethereum address length"));
    }
    let decoded =
        Hex::decode(normalized).map_err(|_| Error::msg("invalid Ethereum address hex format"))?;
    if decoded.len() != 20 {
        return Err(Error::msg("Ethereum address must be 20 bytes"));
    }
    Ok(decoded)
}
