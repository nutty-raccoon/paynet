use nuts::{Amount, nut05::MeltQuoteState};
use starknet_types::{Asset, Unit};

pub trait WithdrawRequest {
    fn asset(&self) -> Asset;
}

pub trait WithdrawAmount {
    fn convert_from(unit: Unit, amount: Amount) -> Self;
}

#[async_trait::async_trait]
pub trait WithdrawInterface: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    type Request: std::fmt::Debug
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + WithdrawRequest
        + Send;
    type Amount: std::fmt::Debug + WithdrawAmount + Send;
    type InvoiceId: Into<[u8; 32]> + Send + Sync + 'static;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::Request, Self::Error>;

    async fn proceed_to_payment(
        &mut self,
        invoice_id: Self::InvoiceId,
        request: Self::Request,
        amount: Self::Amount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Self::Error>;
}
