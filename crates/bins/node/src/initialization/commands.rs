use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ProgramArguments {
    #[arg(long, help = "Path to Starknet configuration file")]
    pub config: Option<PathBuf>,
    #[arg(long, help = "Path to Ethereum configuration file")]
    pub ethereum_config: Option<PathBuf>,
}
