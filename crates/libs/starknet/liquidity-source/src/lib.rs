mod deposit;
#[cfg(not(feature = "mock"))]
mod indexer;
mod init;
mod withdraw;

use std::{
    fmt::{LowerHex, UpperHex},
    num::ParseIntError,
    str::FromStr,
};

pub use deposit::{Depositer, Error as DepositError};
use http::{Uri, uri};
use starknet_types::{CairoShortStringToFeltError, Unit};
use starknet_types_core::{
    felt::{Felt, FromStrError},
    hash::Poseidon,
};
use url::Url;
pub use withdraw::{Error as WithdrawalError, MeltPaymentRequest, Withdrawer};

#[derive(Debug, thiserror::Error)]
pub enum ReadStarknetConfigError {
    #[error("Failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error("Invalid chain id string: {0}")]
    ChainId(#[from] CairoShortStringToFeltError),
    #[error("Invalid felt string: {0}")]
    Felt(#[from] FromStrError),
    #[error("Invalid url string: {0}")]
    Url(#[from] url::ParseError),
    #[error("Invalid uri string: {0}")]
    Uri(#[from] uri::InvalidUri),
    #[error("Invalid block number string: {0}")]
    StartBloc(#[from] ParseIntError),
}

const STARKNET_CASHIER_PRIVATE_KEY_ENV_VAR: &str = "STARKNET_CASHIER_PRIVATE_KEY";
const STARKNET_CHAIN_ID_ENV_VAR: &str = "STARKNET_CHAIN_ID";
const STARKNET_INDEXER_START_BLOCK_ENV_VAR: &str = "STARKNET_INDEXER_START_BLOCK";
const STARKNET_CASHIER_ACCOUNT_ADDRESS_ENV_VAR: &str = "STARKNET_CASHIER_ACCOUNT_ADDRESS";
const STARKNET_SUBSTREAMS_URL_ENV_VAR: &str = "STARKNET_SUBSTREAMS_URL";
const STARKNET_RPC_NODE_URL_ENV_VAR: &str = "STARKNET_RPC_NODE_URL";

pub(crate) fn read_env_variables() -> Result<StarknetCliConfig, ReadStarknetConfigError> {
    let chain_id = std::env::var(STARKNET_CHAIN_ID_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_CHAIN_ID_ENV_VAR, e))?;
    let indexer_start_block = std::env::var(STARKNET_INDEXER_START_BLOCK_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_INDEXER_START_BLOCK_ENV_VAR, e))?;
    let cashier_account_address = std::env::var(STARKNET_CASHIER_ACCOUNT_ADDRESS_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_CASHIER_ACCOUNT_ADDRESS_ENV_VAR, e))?;
    let cashier_private_key = std::env::var(STARKNET_CASHIER_PRIVATE_KEY_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_CASHIER_PRIVATE_KEY_ENV_VAR, e))?;
    let rpc_node_url = std::env::var(STARKNET_RPC_NODE_URL_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_RPC_NODE_URL_ENV_VAR, e))?;
    let substreams_url = std::env::var(STARKNET_SUBSTREAMS_URL_ENV_VAR)
        .map_err(|e| ReadStarknetConfigError::Env(STARKNET_SUBSTREAMS_URL_ENV_VAR, e))?;

    let config = StarknetCliConfig {
        chain_id: starknet_types::ChainId::from_str(&chain_id)?,
        indexer_start_block: indexer_start_block.parse()?,
        cashier_account_address: Felt::from_str(&cashier_account_address)?,
        cashier_private_key: Felt::from_str(&cashier_private_key)?,
        rpc_node_url: Url::from_str(&rpc_node_url)?,
        substreams_url: Uri::from_str(&substreams_url)?,
    };

    Ok(config)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StarknetCliConfig {
    /// The chain we are using as backend
    pub chain_id: starknet_types::ChainId,
    pub indexer_start_block: i64,
    /// The address of the on-chain account managing deposited assets
    pub cashier_account_address: starknet_types_core::felt::Felt,
    pub cashier_private_key: starknet_types_core::felt::Felt,
    /// The url of the starknet rpc node we want to use
    pub rpc_node_url: Url,
    #[serde(with = "uri_serde")]
    pub substreams_url: Uri,
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

mod uri_serde {
    use http::Uri;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        uri.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uri, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uri::from_str(&s).map_err(serde::de::Error::custom)
    }
}
