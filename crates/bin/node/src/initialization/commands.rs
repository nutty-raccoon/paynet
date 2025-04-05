use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ProgramArguments {
    #[arg(long)]
    config: PathBuf,
}

#[cfg(feature = "starknet")]
impl ProgramArguments {
    pub fn read_starknet_config(&self) -> Result<StarknetCliConfig, super::Error> {
        let file_content =
            std::fs::read_to_string(&self.config).map_err(super::Error::CannotReadConfig)?;

        let config: StarknetCliConfig =
            toml::from_str(&file_content).map_err(super::Error::Toml)?;

        Ok(config)
    }
}

#[cfg(feature = "starknet")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StarknetCliConfig {
    /// The chain we are using as backend
    pub chain_id: starknet_types::ChainId,
    /// The address of the on-chain account managing deposited assets
    pub our_account_address: starknet_types_core::felt::Felt,
}
