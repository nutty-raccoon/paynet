use std::path::PathBuf;

use url::Url;

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
