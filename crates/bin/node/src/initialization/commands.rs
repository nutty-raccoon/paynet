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
    pub fn read_starknet_config(&self) -> Result<StarknetUserConfig, super::Error> {
        let file_content =
            std::fs::read_to_string(&self.config).map_err(super::Error::CannotReadConfig)?;

        let config: StarknetUserConfig =
            toml::from_str(&file_content).map_err(super::Error::Toml)?;

        Ok(config)
    }
}

#[cfg(feature = "starknet")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarknetUserConfig {
    /// The chain we are using as backend
    pub chain_id: ChainId,
    /// The address of the on-chain account managing deposited assets
    pub our_account_address: Felt,
}
