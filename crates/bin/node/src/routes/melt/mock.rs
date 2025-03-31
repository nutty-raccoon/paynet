use bitcoin_hashes::Sha256;
use nuts::{Amount, nut05::MeltQuoteState};
use starknet_types::{Asset, Unit};

use super::{MeltBackend, PaymentRequest, errors::Error};

pub struct MockMeltBackend;

impl PaymentRequest for () {
    fn asset(&self) -> starknet_types::Asset {
        Asset::Strk
    }
}

#[async_trait::async_trait]
impl MeltBackend for MockMeltBackend {
    type PaymentRequest = ();

    fn deserialize_payment_request(
        &self,
        _raw_json_string: &str,
    ) -> Result<Self::PaymentRequest, Error> {
        Ok(())
    }

    async fn proceed_to_payment(
        &mut self,
        _quote_hash: Sha256,
        _melt_payment_request: (),
        _unit: Unit,
        _amount: Amount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Error> {
        Ok((MeltQuoteState::Paid, b"cafebabe".to_vec()))
    }
}
