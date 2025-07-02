use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::concurrency::run_concurrency;
use crate::e2e::run_e2e;
use crate::errors::Result;

mod concurrency;
mod e2e;
mod env_variables;
mod errors;
mod utils;

#[derive(clap::Parser)]
#[command(version, about = "Test runner")]
pub struct Cli {
    #[command(subcommand)]
    pub test_type: TestType,
}

#[derive(clap::Subcommand)]
pub enum TestType {
    /// End-to-end tests
    E2e,
    /// Concurrency tests
    Concurrency,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let env = env_variables::read_env_variables()?;

    match cli.test_type {
        TestType::E2e => run_e2e(env).await?,
        TestType::Concurrency => run_concurrency(env).await?,
    }

    Ok(())
}
