use nuts::Amount;
use starknet_types::Unit;
use uuid::Uuid;

pub trait DepositInterface: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    type InvoiceId: Into<[u8; 32]> + Send + Sync + 'static;

    fn generate_deposit_payload(
        &self,
        quote_id: Uuid,
        unit: Unit,
        amount: Amount,
    ) -> Result<(Self::InvoiceId, String), Self::Error>;
}
