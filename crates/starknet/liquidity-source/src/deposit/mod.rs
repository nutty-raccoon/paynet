#[cfg(feature = "mock")]
mod mock;

#[cfg(feature = "mock")]
pub use mock::*;
#[cfg(not(feature = "mock"))]
pub use not_mock::*;

#[cfg(not(feature = "mock"))]
mod not_mock {
    use bitcoin_hashes::Sha256;
    use liquidity_source::DepositInterface;
    use nuts::Amount;
    use starknet_types::{
        Asset, Call, ChainId, Unit, compute_invoice_id, constants::ON_CHAIN_CONSTANTS,
        transactions::generate_single_payment_transaction_calls,
    };
    use starknet_types_core::felt::Felt;
    use uuid::Uuid;

    use crate::StarknetInvoiceId;

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
        type InvoiceId = StarknetInvoiceId;

        fn generate_deposit_payload(
            &self,
            quote_id: Uuid,
            unit: Unit,
            amount: Amount,
            expiry: u64,
        ) -> Result<(Self::InvoiceId, String), Self::Error> {
            let asset = unit.asset();
            let amount = unit.convert_amount_into_u256(amount);
            let on_chain_constants = ON_CHAIN_CONSTANTS.get(self.chain_id.as_str()).unwrap();
            let token_contract_address = on_chain_constants
                .assets_contract_address
                .get_contract_address_for_asset(asset)
                .ok_or(Error::AssetNotFound(asset))?;

            let quote_id_hash =
                Felt::from_bytes_be(Sha256::hash(quote_id.as_bytes()).as_byte_array());
            let calls = generate_single_payment_transaction_calls(
                on_chain_constants.invoice_payment_contract_address,
                quote_id_hash,
                expiry.into(),
                token_contract_address,
                &amount.into(),
                self.our_account_address,
            );
            let calls: Vec<Call> = calls.into_iter().map(Into::into).collect();

            let calls_json_string = serde_json::to_string(&calls)?;

            let invoice_id = compute_invoice_id(quote_id_hash, expiry);

            Ok((StarknetInvoiceId(invoice_id), calls_json_string))
        }
    }
}
