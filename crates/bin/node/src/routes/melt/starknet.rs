use bitcoin_hashes::Sha256;
use nuts::{Amount, nut05::MeltQuoteState};
use starknet_cashier::{StarknetCashierClient, WithdrawRequest};
use starknet_types::{Asset, MeltPaymentRequest, Unit};
use tonic::{Request, transport::Channel};

use super::{MeltBackend, PaymentRequest, errors::Error};

impl PaymentRequest for MeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
}

pub struct StarknetMeltBackend(pub StarknetCashierClient<Channel>);

#[async_trait::async_trait]
impl MeltBackend for StarknetMeltBackend {
    type PaymentRequest = MeltPaymentRequest;

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
        unit: Unit,
        amount: Amount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Error> {
        let amount_to_pay = unit.convert_amount_into_u256(amount);

        let tx_hash = self
            .0
            .withdraw(Request::new(WithdrawRequest {
                invoice_id: quote_hash.to_byte_array().to_vec(),
                asset: melt_payment_request.asset.to_string(),
                amount: amount_to_pay
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
