use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct ProgramArguments {
    #[arg(long)]
    pub config: Option<PathBuf>,
}
