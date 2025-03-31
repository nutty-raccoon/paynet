use bitcoin_hashes::Sha256;
use nuts::Amount;
use starknet_types::{Invoice, StarknetU256, Unit, constants::ON_CHAIN_CONSTANTS};

use crate::app_state::starknet::StarknetConfig;

use super::Error;

pub fn create_starknet_request(
    quote_hash: Sha256,
    unit: Unit,
    amount: Amount,
    config: StarknetConfig,
) -> Result<String, Error> {
    let asset = unit.asset();
    let amount = unit.convert_amount_into_u256(amount);
    let on_chain_constants = ON_CHAIN_CONSTANTS.get(config.chain_id.as_str()).unwrap();

    let json_string = serde_json::to_string(&Invoice {
        id: StarknetU256::from_bytes(quote_hash.as_byte_array()),
        payment_contract_address: on_chain_constants.invoice_payment_contract_address,
        amount,
        token_contract_address: *on_chain_constants
            .assets_contract_address
            .get(asset.as_ref())
            .unwrap(),
        payee: config.our_account_address,
    })
    .map_err(Error::SerQuoteRequest)?;

    Ok(json_string)
}
