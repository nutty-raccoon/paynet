use futures::{FutureExt, select};
use http::Uri;
use sqlx::PgPool;
use starknet_types::ChainId;
use starknet_types_core::felt::Felt;
use tracing::error;

pub async fn init_indexer_task(
    pg_pool: PgPool,
    substreams_endpoint: Uri,
    chain_id: ChainId,
    start_block: i64,
    cashier_account_address: Felt,
) {
    tokio::spawn(async move {
        select! {
          indexer_res =
          substreams_sink::launch(
              pg_pool,
              substreams_endpoint,
              chain_id,
              start_block,
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
}
