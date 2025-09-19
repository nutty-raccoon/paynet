mod deposit;
#[cfg(not(feature = "mock"))]
mod env_config;
#[cfg(not(feature = "mock"))]
mod indexer;
mod init;
mod withdraw;

use std::fmt::{LowerHex, UpperHex};

pub use deposit::{Depositer, Error as DepositError};
use starknet_types::{CairoShortStringToFeltError, Unit};
use starknet_types_core::{felt::Felt, hash::Poseidon};
pub use withdraw::{
    Error as WithdrawalError, MeltPaymentRequest, NewMeltPaymentRequestError, Withdrawer,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(not(feature = "mock"))]
    #[error("failed to init config from env variables: {0}")]
    Config(#[from] env_config::ReadStarknetConfigError),
    #[error("invalid chain id value: {0}")]
    ChainId(CairoShortStringToFeltError),
}

#[derive(Debug, Clone)]
pub struct StarknetInvoiceId(Felt);

impl From<StarknetInvoiceId> for [u8; 32] {
    fn from(value: StarknetInvoiceId) -> Self {
        value.0.to_bytes_be()
    }
}

impl LowerHex for StarknetInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}
impl UpperHex for StarknetInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone)]
pub struct StarknetLiquiditySource {
    pub depositer: Depositer,
    pub withdrawer: Withdrawer,
}

impl liquidity_source::LiquiditySource for StarknetLiquiditySource {
    type Depositer = Depositer;
    type Withdrawer = Withdrawer;
    type InvoiceId = StarknetInvoiceId;
    type Unit = Unit;

    fn depositer(&self) -> Depositer {
        self.depositer.clone()
    }

    fn withdrawer(&self) -> Withdrawer {
        self.withdrawer.clone()
    }

    fn compute_invoice_id(&self, quote_id: uuid::Uuid, expiry: u64) -> Self::InvoiceId {
        let quote_id_hash =
            Felt::from_bytes_be(bitcoin_hashes::Sha256::hash(quote_id.as_bytes()).as_byte_array());
        let mut values = [quote_id_hash, expiry.into(), 2.into()];
        Poseidon::hades_permutation(&mut values);

        StarknetInvoiceId(values[0])
    }
}
