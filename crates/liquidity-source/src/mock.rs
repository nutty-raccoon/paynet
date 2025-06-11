use crate::DepositInterface;
use std::fmt::{LowerHex, UpperHex};

use bitcoin_hashes::Sha256;
use nuts::nut05::MeltQuoteState;
use serde::{Deserialize, Serialize};
use starknet_types::{Asset, StarknetU256};
use starknet_types_core::felt::Felt;
use uuid::Uuid;

use super::{LiquiditySource, WithdrawInterface, WithdrawRequest};

#[derive(Debug, Clone)]
pub struct MockLiquiditySource;

impl LiquiditySource for MockLiquiditySource {
    type Depositer = MockDepositer;
    type Withdrawer = MockWithdrawer;
    type InvoiceId = MockInvoiceId;

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
pub struct Error;

#[derive(Debug, Clone)]
pub struct MockWithdrawer;

impl WithdrawRequest for () {
    fn asset(&self) -> starknet_types::Asset {
        Asset::Strk
    }
    fn amount(&self) -> nuts::Amount {
        nuts::Amount::from(0u64)
    }
}

impl crate::WithdrawAmount for StarknetU256 {
    fn convert_from(unit: starknet_types::Unit, amount: nuts::Amount) -> Self {
        StarknetU256::from(unit.convert_amount_into_u256(amount))
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockMeltPaymentRequest {
    pub payee: Felt,
    pub asset: Asset,
    pub amount: nuts::Amount,
}

impl WithdrawRequest for MockMeltPaymentRequest {
    fn asset(&self) -> Asset {
        self.asset
    }
    fn amount(&self) -> nuts::Amount {
        self.amount
    }
}

#[async_trait::async_trait]
impl WithdrawInterface for MockWithdrawer {
    type Error = Error;
    type Request = MockMeltPaymentRequest;
    type Amount = StarknetU256;
    type InvoiceId = MockInvoiceId;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::Request, Self::Error> {
        if raw_json_string.is_empty() {
            return Ok(MockMeltPaymentRequest {
                payee: Felt::default(),
                asset: Asset::Strk,
                amount: nuts::Amount::from(32u64),
            });
        }
        let pr = serde_json::from_str::<Self::Request>(raw_json_string).unwrap();

        Ok(pr)
    }

    async fn proceed_to_payment(
        &mut self,
        _invoice_id: Uuid,
        _melt_payment_request: Self::Request,
        _amount: Self::Amount,
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
