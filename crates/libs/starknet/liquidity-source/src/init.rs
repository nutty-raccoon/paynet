#[cfg(feature = "mock")]
mod mock_impl {
    use crate::{Depositer, StarknetLiquiditySource, Withdrawer};

    impl StarknetLiquiditySource {
        pub fn new() -> Self {
            StarknetLiquiditySource {
                depositer: Depositer,
                withdrawer: Withdrawer,
            }
        }
    }

    impl Default for StarknetLiquiditySource {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "mock"))]
mod not_mock_impl {
    use std::sync::Arc;

    use sqlx::PgPool;
    use starknet::{
        accounts::{ExecutionEncoding, SingleOwnerAccount},
        providers::{JsonRpcClient, jsonrpc::HttpTransport},
        signers::{LocalWallet, SigningKey},
    };
    use starknet_types::constants::ON_CHAIN_CONSTANTS;

    use crate::{
        Depositer, Error, StarknetLiquiditySource, Withdrawer, indexer, read_env_variables,
    };

    impl StarknetLiquiditySource {
        pub async fn init(pg_pool: PgPool) -> Result<Self, Error> {
            let config = read_env_variables()?;

            // Create provider
            let provider = JsonRpcClient::new(HttpTransport::new(config.rpc_node_url));

            // Create signer
            let signer =
                LocalWallet::from(SigningKey::from_secret_scalar(config.cashier_private_key));

            let account = Arc::new(SingleOwnerAccount::new(
                provider.clone(),
                signer,
                config.cashier_account_address,
                config.chain_id.clone().try_into().map_err(Error::ChainId)?,
                ExecutionEncoding::New,
            ));

            let cloned_chain_id = config.chain_id.clone();
            let cloned_cashier_account_address = config.cashier_account_address;
            let cloned_pg_pool = pg_pool.clone();
            let _handle = tokio::spawn(async move {
                indexer::init_indexer_task(
                    cloned_pg_pool,
                    config.substreams_url,
                    cloned_chain_id,
                    config.indexer_start_block,
                    cloned_cashier_account_address,
                )
                .await
            });

            let on_chain_constants = ON_CHAIN_CONSTANTS.get(config.chain_id.as_str()).unwrap();

            Ok(StarknetLiquiditySource {
                depositer: Depositer::new(config.chain_id.clone(), config.cashier_account_address),
                withdrawer: Withdrawer::new(
                    config.chain_id,
                    account,
                    on_chain_constants.invoice_payment_contract_address,
                ),
            })
        }
    }
}
