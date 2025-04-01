use bitcoin_hashes::Sha256;
use nuts::nut05::MeltQuoteState;
use starknet_cashier::{StarknetCashierClient, WithdrawRequest};
use starknet_types::{Asset, MeltPaymentRequest, StarknetU256, Unit};
use tonic::{Request, transport::Channel};

use super::{MeltBackend, PaymentAmount, PaymentRequest};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid payment request json string: {0}")]
    InvalidPaymentRequest(#[from] serde_json::Error),
    #[error("failed to trigger withdraw from starknet cashier: {0}")]
    StarknetCashier(#[source] tonic::Status),
}

impl PaymentRequest for MeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
}

impl PaymentAmount for StarknetU256 {
    fn convert_from(unit: Unit, amount: nuts::Amount) -> Self {
        unit.convert_amount_into_u256(amount)
    }
}

pub struct StarknetMeltBackend(pub StarknetCashierClient<Channel>);

#[async_trait::async_trait]
impl MeltBackend for StarknetMeltBackend {
    type Error = Error;
    type PaymentRequest = MeltPaymentRequest;
    type PaymentAmount = StarknetU256;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::PaymentRequest, Error> {
        let pr = serde_json::from_str::<Self::PaymentRequest>(raw_json_string)
            .map_err(Error::InvalidPaymentRequest)?;
        Ok(pr)
    }

    async fn proceed_to_payment(
        &mut self,
        quote_hash: Sha256,
        melt_payment_request: MeltPaymentRequest,
        amount: Self::PaymentAmount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Error> {
        let tx_hash = self
            .0
            .withdraw(Request::new(WithdrawRequest {
                invoice_id: quote_hash.to_byte_array().to_vec(),
                asset: melt_payment_request.asset.to_string(),
                amount: amount
                    .to_bytes_be()
                    .into_iter()
                    .skip_while(|&b| b == 0)
                    .collect(),
                payee: melt_payment_request.payee.to_bytes_be().to_vec(),
            }))
            .await
            .map_err(Error::StarknetCashier)?
            .into_inner()
            .tx_hash;

        Ok((MeltQuoteState::Pending, tx_hash))
    }
}
