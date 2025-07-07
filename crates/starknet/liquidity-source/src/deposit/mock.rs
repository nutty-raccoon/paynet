use liquidity_source::DepositInterface;
use starknet_types_core::{felt::Felt, hash::Poseidon};
use uuid::Uuid;

use crate::StarknetInvoiceId;

#[derive(Debug, thiserror::Error)]
#[error("mock liquidity source error")]
pub struct Error;

#[derive(Debug, Clone)]
pub struct Depositer;

impl DepositInterface for Depositer {
    type Error = Error;
    type InvoiceId = StarknetInvoiceId;
    fn generate_deposit_payload(
        &self,
        quote_id: Uuid,
        _unit: starknet_types::Unit,
        _amount: nuts::Amount,
        expiry: u64,
    ) -> Result<(Self::InvoiceId, String), Self::Error> {
        let quote_id_hash =
            Felt::from_bytes_be(bitcoin_hashes::Sha256::hash(quote_id.as_bytes()).as_byte_array());
        let mut values = [quote_id_hash, expiry.into(), 2.into()];
        Poseidon::hades_permutation(&mut values);

        Ok((StarknetInvoiceId(values[0]), "".to_string()))
    }
}
