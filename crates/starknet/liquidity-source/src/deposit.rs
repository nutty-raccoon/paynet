use bitcoin_hashes::Sha256;
use liquidity_source::DepositInterface;
use nuts::Amount;
use starknet_types::{
    Asset, Call, ChainId, Unit, compute_invoice_id, constants::ON_CHAIN_CONSTANTS,
    transactions::generate_payment_transaction_calls,
};
use starknet_types_core::felt::Felt;

#[derive(Debug, Clone)]
pub struct Depositer {
    chain_id: ChainId,
    our_account_address: Felt,
}

impl Depositer {
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

impl DepositInterface for Depositer {
    type Error = Error;
    fn generate_deposit_payload(
        &self,
        quote_hash: Sha256,
        unit: Unit,
        amount: Amount,
        expiry: u64,
    ) -> Result<([u8; 32], String), Self::Error> {
        let asset = unit.asset();
        let amount = unit.convert_amount_into_u256(amount);
        let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
        let token_contract_address = on_chain_constants
            .assets_contract_address
            .get_contract_address_for_asset(asset)
            .ok_or(Error::AssetNotFound(asset))?;

        let quote_id_hash = Felt::from_bytes_be(quote_hash.as_byte_array());
        let calls = generate_payment_transaction_calls(
            token_contract_address,
            on_chain_constants.invoice_payment_contract_address,
            amount.into(),
            quote_id_hash,
            self.our_account_address,
            expiry,
        );
        let calls: Vec<Call> = calls.into_iter().map(Into::into).collect();

        let invoice_id = compute_invoice_id(quote_id_hash, expiry);

        let calls_json_string = serde_json::to_string(&calls)?;

        Ok((invoice_id.to_bytes_be(), calls_json_string))
    }
}
