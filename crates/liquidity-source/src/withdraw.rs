use nuts::{Amount, nut05::MeltQuoteState, traits::Unit};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait WithdrawInterface: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    type Request: std::fmt::Debug + serde::Serialize + for<'de> serde::Deserialize<'de> + Send;
    type Unit: Unit;
    type InvoiceId: Into<[u8; 32]> + Send + Sync + 'static;

    fn compute_total_amount_expected(
        &self,
        request: Self::Request,
        unit: Self::Unit,
        fee: Amount,
    ) -> Result<Amount, Self::Error>;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::Request, Self::Error>;

    async fn proceed_to_payment(
        &mut self,
        quote_id: Uuid,
        request: Self::Request,
        expiry: u64,
    ) -> Result<MeltQuoteState, Self::Error>;
}
