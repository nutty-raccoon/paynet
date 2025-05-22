mod cashier;
mod deposit;
mod indexer;
mod withdraw;

use std::path::PathBuf;

pub use deposit::{Depositer, Error as DepositError};
use sqlx::PgPool;
use starknet_types::ChainId;
use starknet_types_core::felt::Felt;
use tracing::trace;
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
    pub cashier_url: String,
    /// The address of the on-chain account managing deposited assets
    pub cashier_account_address: starknet_types_core::felt::Felt,
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
        pg_pool: PgPool,
        config_path: PathBuf,
    ) -> Result<StarknetLiquiditySource, Error> {
        let config = read_starknet_config(config_path)?;

        let apibara_token = match config.chain_id {
            // Not needed for local DNA service
            ChainId::Devnet => "".to_string(),
            _ => std::env::var("APIBARA_TOKEN").map_err(|e| Error::Env("APIBARA_TOKEN", e))?,
        };

        let cashier = cashier::connect(config.cashier_url.clone(), &config.chain_id).await?;
        trace!("connected to starknet cashier server");

        let cloned_chain_id = config.chain_id.clone();
        let cloned_cashier_account_address = config.cashier_account_address;
        let _handle = tokio::spawn(async move {
            indexer::run_in_ctrl_c_cancellable_task(
                pg_pool,
                apibara_token,
                cloned_chain_id,
                cloned_cashier_account_address,
            )
            .await
        });

        Ok(StarknetLiquiditySource {
            depositer: Depositer::new(config.chain_id, config.cashier_account_address),
            withdrawer: Withdrawer(cashier),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StarknetInvoiceId(Felt);

impl From<StarknetInvoiceId> for [u8; 32] {
    fn from(value: StarknetInvoiceId) -> Self {
        value.0.to_bytes_be()
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
    type InvoiceId = StarknetInvoiceId;

    fn depositer(&self) -> Depositer {
        self.depositer.clone()
    }

    fn withdrawer(&self) -> Withdrawer {
        self.withdrawer.clone()
    }

    fn compute_invoice_id(&self, quote_id: uuid::Uuid) -> Self::InvoiceId {
        StarknetInvoiceId(Felt::from_bytes_be(
            bitcoin_hashes::Sha256::hash(quote_id.as_bytes()).as_byte_array(),
        ))
    }
}
