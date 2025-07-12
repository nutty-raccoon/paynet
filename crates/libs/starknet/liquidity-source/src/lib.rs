mod deposit;
#[cfg(not(feature = "mock"))]
mod indexer;
mod init;
mod withdraw;

use std::{
    fmt::{LowerHex, UpperHex},
    path::PathBuf,
};

pub use deposit::{Depositer, Error as DepositError};
use starknet_types::{CairoShortStringToFeltError, Unit};
use starknet_types_core::{felt::Felt, hash::Poseidon};
use url::Url;
pub use withdraw::{Error as WithdrawalError, MeltPaymentRequest, Withdrawer};

#[derive(Debug, thiserror::Error)]
pub enum ReadStarknetConfigError {
    #[error("failed to read Starknet config file: {0}")]
    IO(#[from] std::io::Error),
    #[error("failed to deserialize Starknet config file content: {0}")]
    Toml(#[from] toml::de::Error),
}

pub fn read_starknet_config(path: PathBuf) -> Result<StarknetCliConfig, ReadStarknetConfigError> {
    let file_content = std::fs::read_to_string(&path)?;

    let config: StarknetCliConfig = toml::from_str(&file_content)?;

    Ok(config)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StarknetCliConfig {
    /// The chain we are using as backend
    pub chain_id: starknet_types::ChainId,
    /// The address of the on-chain account managing deposited assets
    pub cashier_account_address: starknet_types_core::felt::Felt,
    /// The url of the starknet rpc node we want to use
    pub starknet_rpc_node_url: Url,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    Config(#[from] ReadStarknetConfigError),
    #[cfg(not(feature = "mock"))]
    #[error(transparent)]
    Indexer(#[from] indexer::Error),
    #[error("invalid private key value")]
    PrivateKey,
    #[error("invalid chain id value: {0}")]
    ChainId(CairoShortStringToFeltError),
}

pub const CASHIER_PRIVATE_KEY_ENV_VAR: &str = "CASHIER_PRIVATE_KEY";

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
