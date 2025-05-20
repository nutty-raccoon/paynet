use bitcoin_hashes::Sha256;
use nuts::nut05::MeltQuoteState;
use starknet_types::{Asset, StarknetU256};

use crate::DepositInterface;

use super::{LiquiditySource, WithdrawInterface, WithdrawRequest};

#[derive(Debug, Clone)]
pub struct MockLiquiditySource;

impl LiquiditySource for MockLiquiditySource {
    type Depositer = MockDepositer;
    type Withdrawer = MockWithdrawer;

    fn depositer(&self) -> MockDepositer {
        MockDepositer
    }

    fn withdrawer(&self) -> MockWithdrawer {
        MockWithdrawer
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
}

impl crate::WithdrawAmount for StarknetU256 {
    fn convert_from(unit: starknet_types::Unit, amount: nuts::Amount) -> Self {
        StarknetU256::from(unit.convert_amount_into_u256(amount))
    }
}

#[async_trait::async_trait]
impl WithdrawInterface for MockWithdrawer {
    type Error = Error;
    type Request = ();
    type Amount = StarknetU256;

    fn deserialize_payment_request(
        &self,
        _raw_json_string: &str,
    ) -> Result<Self::Request, Self::Error> {
        Ok(())
    }

    async fn proceed_to_payment(
        &mut self,
        _quote_hash: Sha256,
        _melt_payment_request: (),
        _amount: Self::Amount,
        _ttl: u64,
    ) -> Result<(MeltQuoteState, Vec<u8>), Self::Error> {
        Ok((MeltQuoteState::Paid, "caffebabe".as_bytes().to_vec()))
    }
}

#[derive(Debug, Clone)]
pub struct MockDepositer;

#[async_trait::async_trait]
impl DepositInterface for MockDepositer {
    type Error = Error;

    async fn generate_deposit_payload(
        &self,
        quote_hash: Sha256,
        _unit: starknet_types::Unit,
        _amount: nuts::Amount,
        _expiry: u64,
    ) -> Result<([u8; 32], String), Self::Error> {
        Ok((quote_hash.to_byte_array(), "".to_string()))
    }
}
