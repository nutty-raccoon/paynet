#[cfg(feature = "mock")]
pub mod mock;
#[cfg(feature = "starknet")]
pub mod starknet;

use bitcoin_hashes::Sha256;
use nuts::{Amount, nut05::MeltQuoteState};
use starknet_types::{Asset, Unit};

pub trait PaymentRequest {
    fn asset(&self) -> Asset;
}

pub trait PaymentAmount {
    fn convert_from(unit: Unit, amount: Amount) -> Self;
}

#[async_trait::async_trait]
pub trait MeltBackend {
    type Error: std::error::Error + Send + Sync;
    type PaymentRequest: std::fmt::Debug
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + PaymentRequest
        + Send;
    type PaymentAmount: std::fmt::Debug + PaymentAmount + Send;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::PaymentRequest, Self::Error>;

    async fn proceed_to_payment(
        &mut self,
        quote_hash: Sha256,
        request: Self::PaymentRequest,
        amount: Self::PaymentAmount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Self::Error>;
}
