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
    use std::{path::PathBuf, str::FromStr, sync::Arc};

    use sqlx::PgPool;
    use starknet::{
        accounts::{ExecutionEncoding, SingleOwnerAccount},
        providers::{JsonRpcClient, jsonrpc::HttpTransport},
        signers::{LocalWallet, SigningKey},
    };
    use starknet_types::{ChainId, constants::ON_CHAIN_CONSTANTS};
    use starknet_types_core::felt::Felt;

    use crate::{
        CASHIER_PRIVATE_KEY_ENV_VAR, Depositer, Error, StarknetLiquiditySource, Withdrawer,
        indexer, read_starknet_config,
    };

    impl StarknetLiquiditySource {
        pub async fn init(pg_pool: PgPool, config_path: PathBuf) -> Result<Self, Error> {
            let config = read_starknet_config(config_path)?;
            let private_key = Felt::from_str(
                &std::env::var(CASHIER_PRIVATE_KEY_ENV_VAR)
                    .map_err(|e| Error::Env(CASHIER_PRIVATE_KEY_ENV_VAR, e))?,
            )
            .map_err(|_| Error::PrivateKey)?;

            let apibara_token = match config.chain_id {
                // Not needed for local DNA service
                ChainId::Devnet => "".to_string(),
                _ => std::env::var("APIBARA_TOKEN").map_err(|e| Error::Env("APIBARA_TOKEN", e))?,
            };

            // Create provider
            let provider = JsonRpcClient::new(HttpTransport::new(config.starknet_rpc_node_url));

            // Create signer
            let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));

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
                indexer::run_in_ctrl_c_cancellable_task(
                    cloned_pg_pool,
                    apibara_token,
                    cloned_chain_id,
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
