use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;
use thiserror::Error;

use crate::errors::InitializationError;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long)]
    config: PathBuf,
}

#[cfg(feature = "indexer")]
impl Args {
    pub fn read_config(&self) -> Result<Config, InitializationError> {
        let file_content =
            std::fs::read_to_string(&self.config).map_err(InitializationError::CannotReadConfig)?;

        let config: ConfigFileContent =
            toml::from_str(&file_content).map_err(InitializationError::Toml)?;

        config.try_into().map_err(InitializationError::Config)
    }
}

/// The chain where the represented assets live
#[derive(Debug, Clone, Copy)]
pub enum ChainId {
    /// Starknet mainnet
    Mainnet,
    /// Starknet sepolia testnet
    Sepolia,
    /// A custom network
    ///
    /// The inner value should be a valid cairo short string, otherwise IO will panic
    Custom(Felt),
}

impl ChainId {
    pub fn new_custom(s: &str) -> Result<Self, starknet_types::CairoShortStringToFeltError> {
        let short_string = starknet_types::felt_from_short_string(s)?;

        Ok(Self::Custom(short_string))
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::Mainnet => std::fmt::Display::fmt("mainnet", f),
            ChainId::Sepolia => std::fmt::Display::fmt("sepolia", f),
            ChainId::Custom(felt) => {
                let as_short_string =
                    starknet_types::felt_to_short_string(*felt).map_err(|_| std::fmt::Error)?;
                std::fmt::Display::fmt(&as_short_string, f)
            }
        }
    }
}

impl Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_string = self.to_string();

        serializer.serialize_str(&as_string)
    }
}

impl<'de> Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let short_string = <String>::deserialize(deserializer)?;
        match short_string.as_str() {
            "mainnet" => Ok(ChainId::Mainnet),
            "sepolia" => Ok(ChainId::Sepolia),
            s => ChainId::new_custom(s).map_err(|_| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(s),
                    &"a valid cairo short string",
                )
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFileContent {
    /// The chain we are using as backend
    chain_id: ChainId,
    /// The address of the STRK token address
    ///
    /// Optional if chain_id is "mainnet" or "sepolia"
    strk_address: Option<Felt>,
    /// The address of the on-chain account managing deposited assets
    recipient_address: Felt,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The chain we are using as backend
    pub chain_id: ChainId,
    /// The address of the STRK token address
    pub strk_address: Felt,
    /// The address of the on-chain account managing deposited assets
    pub recipient_address: Felt,
}

const MAINNET_STRK_TOKEN_CONTRACT: Felt =
    Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d");
const SEPOLIA_STRK_TOKEN_CONTRACT: Felt =
    Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d");

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Cannot specify custom STRK contract Address for chain {0}")]
    CannotSpecifyCustomContractAddressForChainId(ChainId),
    #[error("Must specify custom STRK contract Address for custom chains")]
    MustSpecifyCustomContractAddressForCustom,
}

impl TryFrom<ConfigFileContent> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFileContent) -> Result<Self, Self::Error> {
        let token_address = match (value.chain_id, value.strk_address) {
            (ChainId::Mainnet, None) => Ok(MAINNET_STRK_TOKEN_CONTRACT),
            (ChainId::Sepolia, None) => Ok(SEPOLIA_STRK_TOKEN_CONTRACT),
            (ChainId::Custom(_), Some(f)) => Ok(f),
            (ChainId::Custom(_), None) => {
                Err(ConfigError::MustSpecifyCustomContractAddressForCustom)
            }
            (chain_id, Some(_)) => Err(ConfigError::CannotSpecifyCustomContractAddressForChainId(
                chain_id,
            )),
        }?;

        Ok(Config {
            chain_id: value.chain_id,
            strk_address: token_address,
            recipient_address: value.recipient_address,
        })
    }
}

pub fn read_env_variables() -> Result<EnvVariables, InitializationError> {
    // Only if we are in debug mode, we allow loading env variable from a .env file
    #[cfg(debug_assertions)]
    dotenvy::from_filename("node.env").map_err(InitializationError::Dotenvy)?;

    #[cfg(feature = "indexer")]
    let apibara_token = std::env::var("APIBARA_TOKEN").map_err(InitializationError::Env)?;
    let pg_url = std::env::var("PG_URL").map_err(InitializationError::Env)?;
    let signer_url = std::env::var("SIGNER_URL").map_err(InitializationError::Env)?;
    let grpc_ip = std::env::var("GRPC_IP").map_err(InitializationError::Env)?;
    let grpc_port = std::env::var("GRPC_PORT")
        .map_err(InitializationError::Env)?
        .parse()
        .map_err(InitializationError::ParseInt)?;

    Ok(EnvVariables {
        #[cfg(feature = "indexer")]
        apibara_token,
        pg_url,
        signer_url,
        grpc_ip,
        grpc_port,
    })
}

#[derive(Debug)]
pub struct EnvVariables {
    #[cfg(feature = "indexer")]
    pub apibara_token: String,
    pub pg_url: String,
    pub signer_url: String,
    pub grpc_ip: String,
    pub grpc_port: u16,
}
