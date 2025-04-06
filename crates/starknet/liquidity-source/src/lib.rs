mod cashier;
mod deposit;
mod indexer;
mod withdraw;

use std::path::PathBuf;

pub use deposit::{Depositer, Error as DepositError};
use log::info;
use sqlx::{Postgres, pool::PoolConnection};
pub use withdraw::{
    Error as WithdrawalError, MeltPaymentRequest, StarknetU256WithdrawAmount, Withdrawer,
};

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
    pub our_account_address: starknet_types_core::felt::Felt,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    Config(#[from] ReadStarknetConfigError),
    #[error(transparent)]
    Cashier(#[from] cashier::Error),
    #[error(transparent)]
    Indexer(#[from] indexer::Error),
}

impl StarknetLiquiditySource {
    pub async fn init(
        conn: PoolConnection<Postgres>,
        config_path: PathBuf,
    ) -> Result<StarknetLiquiditySource, Error> {
        let cashier_url = std::env::var("CASHIER_URL").map_err(|e| Error::Env("CASHIER_URL", e))?;
        let apibara_token =
            std::env::var("APIBARA_TOKEN").map_err(|e| Error::Env("APIBARA_TOKEN", e))?;

        let config = read_starknet_config(config_path)?;

        let cashier = cashier::connect(cashier_url, &config.chain_id).await?;
        info!("Connected to starknet cashier server.");

        indexer::run_in_ctrl_c_cancellable_task(conn, apibara_token, &config).await?;

        Ok(StarknetLiquiditySource {
            depositer: Depositer::new(config.chain_id, config.our_account_address),
            withdrawer: Withdrawer(cashier),
        })
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

    fn depositer(&self) -> Depositer {
        self.depositer.clone()
    }

    fn withdrawer(&self) -> Withdrawer {
        self.withdrawer.clone()
    }
}
