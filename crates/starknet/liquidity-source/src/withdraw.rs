use nuts::nut05::MeltQuoteState;
use serde::{Deserialize, Serialize};
use starknet_cashier::{StarknetCashierClient, WithdrawRequest as CashierWithdrawRequest};
use starknet_types::{Asset, StarknetU256, Unit};
use starknet_types_core::felt::Felt;
use tonic::{Request, transport::Channel};

use liquidity_source::{WithdrawAmount, WithdrawInterface, WithdrawRequest};
use starknet_types::is_valid_starknet_address;
use uuid::Uuid;

use crate::StarknetInvoiceId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid payment request json string: {0}")]
    InvalidPaymentRequest(#[from] serde_json::Error),
    #[error("failed to trigger withdraw from starknet cashier: {0}")]
    StarknetCashier(#[source] tonic::Status),
    #[error("invalid starknet address: {0}")]
    InvalidStarknetAddress(Felt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeltPaymentRequest {
    pub payee: Felt,
    pub asset: Asset,
}

impl WithdrawRequest for MeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct StarknetU256WithdrawAmount(pub StarknetU256);

impl WithdrawAmount for StarknetU256WithdrawAmount {
    fn convert_from(unit: Unit, amount: nuts::Amount) -> Self {
        Self(StarknetU256::from(unit.convert_amount_into_u256(amount)))
    }
}

#[derive(Debug, Clone)]
pub struct Withdrawer(pub StarknetCashierClient<tower_otel::trace::Grpc<Channel>>);

impl Withdrawer {
    pub fn new(cashier: StarknetCashierClient<tower_otel::trace::Grpc<Channel>>) -> Self {
        Self(cashier)
    }
}

#[async_trait::async_trait]
impl WithdrawInterface for Withdrawer {
    type Error = Error;
    type Request = MeltPaymentRequest;
    type Amount = StarknetU256WithdrawAmount;
    type InvoiceId = StarknetInvoiceId;

    fn deserialize_payment_request(&self, raw_json_string: &str) -> Result<Self::Request, Error> {
        let pr = serde_json::from_str::<Self::Request>(raw_json_string)
            .map_err(Error::InvalidPaymentRequest)?;

        if !is_valid_starknet_address(&pr.payee) {
            return Err(Error::InvalidStarknetAddress(pr.payee));
        }
        Ok(pr)
    }

    async fn proceed_to_payment(
        &mut self,
        quote_id: Uuid,
        melt_payment_request: MeltPaymentRequest,
        amount: Self::Amount,
        expiry: u64,
    ) -> Result<(MeltQuoteState, Self::InvoiceId), Error> {
        // let tx_hash = self
        //     .0
        //     .withdraw(Request::new(CashierWithdrawRequest {
        //         invoice_id: invoice_id.0.to_bytes_be().to_vec(),
        //         asset: melt_payment_request.asset.to_string(),
        //         amount: amount
        //             .0
        //             .to_bytes_be()
        //             .into_iter()
        //             .skip_while(|&b| b == 0)
        //             .collect(),
        //         payee: melt_payment_request.payee.to_bytes_be().to_vec(),
        //         expiry,
        //     }))
        //     .await
        //     .map_err(Error::StarknetCashier)?
        //     .into_inner()
        //     .tx_hash;
        //
        let invoice_id = todo!();

        Ok((MeltQuoteState::Pending, invoice_id))
    }
}
