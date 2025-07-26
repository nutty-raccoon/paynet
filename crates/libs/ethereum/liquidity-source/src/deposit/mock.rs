use liquidity_source::DepositInterface;
use primitive_types::H256;
use uuid::Uuid;

use crate::EthereumInvoiceId;

#[derive(Debug, thiserror::Error)]
#[error("mock liquidity source error")]
pub struct Error;

#[derive(Debug, Clone)]
pub struct Depositer;

impl DepositInterface for Depositer {
    type Error = Error;
    type InvoiceId = EthereumInvoiceId;
    
    fn generate_deposit_payload(
        &self,
        quote_id: Uuid,
        _unit: ethereum_types::Unit,
        _amount: nuts::Amount,
        expiry: u64,
    ) -> Result<(Self::InvoiceId, String), Self::Error> {
        let quote_id_hash = bitcoin_hashes::sha256::Hash::hash(quote_id.as_bytes());
        
        // Create a deterministic hash combining quote_id, expiry, and a chain identifier
        let mut hasher = bitcoin_hashes::sha256::HashEngine::default();
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, quote_id_hash.as_byte_array());
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, &expiry.to_be_bytes());
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, &[1u8]); // Ethereum chain identifier
        
        let final_hash = bitcoin_hashes::sha256::Hash::from_engine(hasher);
        let invoice_id = EthereumInvoiceId(H256::from_slice(final_hash.as_byte_array()));

        Ok((invoice_id, "".to_string()))
    }
}
