use bitcoin_hashes::Sha256;
use nuts::Amount;
use starknet_types::Unit;

pub trait DepositInterface: Send {
    type Error: std::error::Error + Send + Sync + 'static;

    fn generate_deposit_payload(
        &self,
        quote_hash: Sha256,
        unit: Unit,
        amount: Amount,
    ) -> Result<([u8; 32], String), Self::Error>;
}
