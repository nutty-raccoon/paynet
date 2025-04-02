use bitcoin_hashes::Sha256;
use nuts::Amount;
use serde::{Deserialize, Serialize};
use starknet_types::{
    Asset, ChainId, StarknetU256, Unit, constants::ON_CHAIN_CONSTANTS,
    transactions::generate_payment_transaction_calls,
};
use starknet_types_core::felt::Felt;

use crate::DepositInterface;

#[derive(Debug, Clone)]
pub struct StarknetDepositer {
    chain_id: ChainId,
    our_account_address: Felt,
}

impl StarknetDepositer {
    pub fn new(chain_id: ChainId, our_account_address: Felt) -> Self {
        Self {
            chain_id,
            our_account_address,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("asset {0} not found in on-chain constants")]
    AssetNotFound(Asset),
    #[error("failed to serialize Calls: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[async_trait::async_trait]
impl DepositInterface for StarknetDepositer {
    type Error = Error;

    fn generate_deposit_payload(
        &self,
        quote_hash: Sha256,
        unit: Unit,
        amount: Amount,
    ) -> Result<String, Self::Error> {
        let asset = unit.asset();
        let amount = unit.convert_amount_into_u256(amount);
        let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
        let token_contract_address = *on_chain_constants
            .assets_contract_address
            .get(asset.as_str())
            .ok_or(Error::AssetNotFound(asset))?;

        let calls = generate_payment_transaction_calls(
            token_contract_address,
            on_chain_constants.invoice_payment_contract_address,
            amount,
            StarknetU256::from_bytes(quote_hash.as_byte_array()),
            self.our_account_address,
        );
        let calls: Vec<Call> = calls
            .into_iter()
            .map(|c| Call {
                to: c.to,
                selector: c.selector,
                calldata: c.calldata,
            })
            .collect();

        let calls_json_string = serde_json::to_string(&calls)?;

        Ok(calls_json_string)
    }
}

// TODO: remove and use starknet-core struct when https://github.com/xJonathanLEI/starknet-rs/pull/713 is merged
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Call {
    /// Address of the contract being invoked.
    pub to: Felt,
    /// Entrypoint selector of the function being invoked.
    pub selector: Felt,
    /// List of calldata to be sent for the call.
    pub calldata: Vec<Felt>,
}
