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
use http::Uri;
use ethereum_types::{Unit, EthereumAddress, ChainId};
use primitive_types::H256;
use url::Url;
pub use withdraw::{Error as WithdrawalError, MeltPaymentRequest, Withdrawer};

#[derive(Debug, thiserror::Error)]
pub enum ReadEthereumConfigError {
    #[error("failed to read Ethereum config file: {0}")]
    IO(#[from] std::io::Error),
    #[error("failed to deserialize Ethereum config file content: {0}")]
    Toml(#[from] toml::de::Error),
}

pub fn read_ethereum_config(path: PathBuf) -> Result<EthereumCliConfig, ReadEthereumConfigError> {
    let file_content = std::fs::read_to_string(&path)?;
    let config: EthereumCliConfig = toml::from_str(&file_content)?;
    Ok(config)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EthereumCliConfig {
    /// The chain we are using as backend
    pub chain_id: ChainId,
    /// The address of the on-chain account managing deposited assets
    pub cashier_account_address: EthereumAddress,
    /// The url of the ethereum rpc node we want to use
    pub ethereum_rpc_node_url: Url,
    #[serde(with = "uri_serde")]
    pub ethereum_substreams_url: Uri,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    Config(#[from] ReadEthereumConfigError),
    #[cfg(not(feature = "mock"))]
    #[error(transparent)]
    Indexer(#[from] indexer::Error),
    #[error("invalid private key value")]
    PrivateKey,
    #[error("invalid chain id value: {0}")]
    ChainId(String),
}

pub const CASHIER_PRIVATE_KEY_ENV_VAR: &str = "ETHEREUM_CASHIER_PRIVATE_KEY";

#[derive(Debug, Clone)]
pub struct EthereumInvoiceId(H256);

impl From<EthereumInvoiceId> for [u8; 32] {
    fn from(value: EthereumInvoiceId) -> Self {
        value.0.0
    }
}

impl LowerHex for EthereumInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl UpperHex for EthereumInvoiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone)]
pub struct EthereumLiquiditySource {
    pub depositer: Depositer,
    pub withdrawer: Withdrawer,
}

impl liquidity_source::LiquiditySource for EthereumLiquiditySource {
    type Depositer = Depositer;
    type Withdrawer = Withdrawer;
    type InvoiceId = EthereumInvoiceId;
    type Unit = Unit;

    fn depositer(&self) -> Depositer {
        self.depositer.clone()
    }

    fn withdrawer(&self) -> Withdrawer {
        self.withdrawer.clone()
    }

    fn compute_invoice_id(&self, quote_id: uuid::Uuid, expiry: u64) -> Self::InvoiceId {
        // Use Keccak256 hash for Ethereum (similar to how Starknet uses Poseidon)
        let quote_id_hash = bitcoin_hashes::sha256::Hash::hash(quote_id.as_bytes());
        
        // Create a deterministic hash combining quote_id, expiry, and a chain identifier
        let mut hasher = bitcoin_hashes::sha256::HashEngine::default();
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, quote_id_hash.as_byte_array());
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, &expiry.to_be_bytes());
        bitcoin_hashes::Hash::engine_input_all(&mut hasher, &[1u8]); // Ethereum chain identifier
        
        let final_hash = bitcoin_hashes::sha256::Hash::from_engine(hasher);
        EthereumInvoiceId(H256::from_slice(final_hash.as_byte_array()))
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
