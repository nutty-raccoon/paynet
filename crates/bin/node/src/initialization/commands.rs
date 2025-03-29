use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};
use starknet_types::ChainId;
use starknet_types_core::felt::Felt;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ProgramArguments {
    #[arg(long)]
    config: PathBuf,
}

#[cfg(feature = "starknet")]
impl ProgramArguments {
    pub fn read_starknet_config(&self) -> Result<StarknetConfig, super::Error> {
        let file_content =
            std::fs::read_to_string(&self.config).map_err(super::Error::CannotReadConfig)?;

        let config: StarknetConfig = toml::from_str(&file_content).map_err(super::Error::Toml)?;

        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StarknetConfig {
    /// The chain we are using as backend
    pub chain_id: ChainId,
    /// The address of the on-chain account managing deposited assets
    pub recipient_address: Felt,
}
