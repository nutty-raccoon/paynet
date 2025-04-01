use bitcoin_hashes::Sha256;
use nuts::nut05::MeltQuoteState;
use starknet_types::{Asset, StarknetU256};

use super::{MeltBackend, PaymentRequest};

#[derive(Debug, thiserror::Error)]
#[error("mock liquidity source error")]
pub struct Error;

pub struct MockMeltBackend;

impl PaymentRequest for () {
    fn asset(&self) -> starknet_types::Asset {
        Asset::Strk
    }
}

#[async_trait::async_trait]
impl MeltBackend for MockMeltBackend {
    type Error = Error;
    type PaymentRequest = ();
    type PaymentAmount = StarknetU256;

    fn deserialize_payment_request(
        &self,
        _raw_json_string: &str,
    ) -> Result<Self::PaymentRequest, Self::Error> {
        Ok(())
    }

    async fn proceed_to_payment(
        &mut self,
        _quote_hash: Sha256,
        _melt_payment_request: (),
        _amount: Self::PaymentAmount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Self::Error> {
        Ok((MeltQuoteState::Paid, "caffebabe".as_bytes().to_vec()))
    }
}
