use crate::DepositInterface;
use std::fmt::{LowerHex, UpperHex};

use bitcoin_hashes::Sha256;
use nuts::{Amount, nut05::MeltQuoteState};
use serde::{Deserialize, Serialize};
use starknet_types::Unit;
use uuid::Uuid;

use super::{LiquiditySource, WithdrawInterface};

#[derive(Debug, Clone)]
pub struct MockLiquiditySource;

impl LiquiditySource for MockLiquiditySource {
    type Depositer = MockDepositer;
    type Withdrawer = MockWithdrawer;
    type InvoiceId = MockInvoiceId;
    type Unit = Unit;

    fn depositer(&self) -> MockDepositer {
        MockDepositer
    }

    fn withdrawer(&self) -> MockWithdrawer {
        MockWithdrawer
    }

    fn compute_invoice_id(&self, quote_id: Uuid, _expiry: u64) -> Self::InvoiceId {
        MockInvoiceId(Sha256::hash(quote_id.as_bytes()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("mock liquidity source error")]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct MockWithdrawer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockWithdrawRequest {
    amount: Amount,
}

#[derive(Debug, Clone)]
pub struct MockInvoiceId(Sha256);

impl From<MockInvoiceId> for [u8; 32] {
    fn from(value: MockInvoiceId) -> Self {
        value.0.to_byte_array()
    }
}

impl LowerHex for MockInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}
impl UpperHex for MockInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

#[async_trait::async_trait]
impl WithdrawInterface for MockWithdrawer {
    type Error = Error;
    type Request = MockWithdrawRequest;
    type InvoiceId = MockInvoiceId;
    type Unit = Unit;

    fn compute_total_amount_expected(
        &self,
        request: Self::Request,
        _unit: Self::Unit,
        fee: Amount,
    ) -> Result<Amount, Self::Error> {
        Ok(request.amount + fee)
    }

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::Request, Self::Error> {
        let request = serde_json::from_str(raw_json_string)?;
        Ok(request)
    }

    async fn proceed_to_payment(
        &mut self,
        _invoice_id: Uuid,
        _melt_payment_request: Self::Request,
        _expiry: u64,
    ) -> Result<MeltQuoteState, Self::Error> {
        Ok(MeltQuoteState::Paid)
    }
}

#[derive(Debug, Clone)]
pub struct MockDepositer;

impl DepositInterface for MockDepositer {
    type Error = Error;
    type InvoiceId = MockInvoiceId;

    fn generate_deposit_payload(
        &self,
        quote_id: Uuid,
        _unit: starknet_types::Unit,
        _amount: nuts::Amount,
        _expiry: u64,
    ) -> Result<(Self::InvoiceId, String), Self::Error> {
        Ok((
            MockInvoiceId(bitcoin_hashes::Sha256::hash(quote_id.as_bytes())),
            "".to_string(),
        ))
    }
}
