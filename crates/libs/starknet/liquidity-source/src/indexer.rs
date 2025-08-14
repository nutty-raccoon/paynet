use futures::{FutureExt, select};
use http::Uri;
use sqlx::PgPool;
use starknet_types::{AssetToUnitConversionError, ChainId};
use starknet_types_core::felt::Felt;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to open connection with sqlite db: {0}")]
    OpenSqlite(#[source] rusqlite::Error),
    #[error("unknown chain id: {0}")]
    UnknownChainId(ChainId),
    #[error("failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error("failed to interact with the indexer: {0}")]
    Indexer(#[from] anyhow::Error),
    #[error("failed to interact with the node database: {0}")]
    DbNode(#[from] db_node::Error),
    #[error("failed to interact with the node database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Felt(#[from] starknet_types_core::felt::FromStrError),
    #[error("u256 value overflowed during the computation of the total amount paid for invoice")]
    AmountPaidOverflow,
    #[error(transparent)]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
}

pub async fn init_indexer_task(
    pg_pool: PgPool,
    substreams_endpoint: Uri,
    chain_id: ChainId,
    cashier_account_address: Felt,
) -> Result<(), Error> {
    tokio::spawn(async move {
        select! {
          indexer_res =
          starknet_substreams_sink::launch(
              pg_pool,
              substreams_endpoint,
              chain_id,
              cashier_account_address,
          ).fuse() => match indexer_res {
                Ok(()) => {
                    error!(name: "indexer-task-error", name = "indexer-task-error", error = "returned");
                },
                Err(err) => {
                    error!(name: "indexer-task-error", name = "indexer-task-error", error = ?err);
                },

          },
          sig = tokio::signal::ctrl_c().fuse() => match sig {
                Ok(()) => {},
                Err(err) => {
                    error!(name: "ctrl-c-error", name = "ctrl-c-error", error = ?err);
                },
            }
        };
    });

    Ok(())
}
