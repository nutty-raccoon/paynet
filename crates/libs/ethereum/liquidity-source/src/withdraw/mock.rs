use async_trait::async_trait;
use ethereum_types::{Unit, is_valid_ethereum_address};
use liquidity_source::WithdrawInterface;
use nuts::{Amount, nut05::MeltQuoteState};
use uuid::Uuid;

use crate::EthereumInvoiceId;
use super::MeltPaymentRequest;

#[derive(Debug, thiserror::Error)]
#[error("mock liquidity source error")]
pub struct Error;

#[derive(Debug, Clone)]
pub struct Withdrawer;

#[async_trait::async_trait]
impl WithdrawInterface for Withdrawer {
    type Error = Error;
    type Request = MeltPaymentRequest;
    type Unit = Unit;
    type InvoiceId = EthereumInvoiceId;

    fn compute_total_amount_expected(
        &self,
        request: Self::Request,
        unit: Self::Unit,
        fee: Amount,
    ) -> Result<Amount, Self::Error> {
        let fee_u256 = unit.convert_amount_into_u256(fee);
        let total_amount = request.amount + fee_u256;

        unit.convert_u256_into_amount(total_amount)
            .map_err(|_| Error)
    }

    fn deserialize_payment_request(&self, raw_json_string: &str) -> Result<Self::Request, Error> {
        let pr = serde_json::from_str::<Self::Request>(raw_json_string)
            .map_err(|_| Error)?;

        if !is_valid_ethereum_address(&format!("{:?}", pr.payee)) {
            return Err(Error);
        }

        Ok(pr)
    }

    async fn proceed_to_payment(
        &mut self,
        _quote_id: Uuid,
        _melt_payment_request: MeltPaymentRequest,
        _expiry: u64,
    ) -> Result<MeltQuoteState, Error> {
        Ok(MeltQuoteState::Paid)
    }
}
