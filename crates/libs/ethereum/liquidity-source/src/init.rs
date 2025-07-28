#[cfg(feature = "mock")]
mod mock {
    use crate::{EthereumLiquiditySource, deposit::Depositer, withdraw::Withdrawer};

    impl EthereumLiquiditySource {
        pub fn new() -> Self {
            Self {
                depositer: Depositer,
                withdrawer: Withdrawer,
            }
        }
    }
}

#[cfg(not(feature = "mock"))]
mod not_mock {
    use std::{path::PathBuf, sync::Arc, str::FromStr};
    use sqlx::PgPool;
    use tokio::sync::mpsc;
    use ethers::{
        providers::{Http, Provider},
        signers::{LocalWallet, Signer},
        middleware::SignerMiddleware,
    };
    use ethereum_types::{ChainId, EthereumAddress, constants::ON_CHAIN_CONSTANTS};

    use crate::{
        EthereumLiquiditySource, Error, CASHIER_PRIVATE_KEY_ENV_VAR,
        read_ethereum_config, deposit::Depositer, withdraw::Withdrawer,
    };

    #[cfg(not(feature = "mock"))]
    use crate::indexer;

    type EthereumClient = SignerMiddleware<Provider<Http>, LocalWallet>;

    impl EthereumLiquiditySource {
        pub async fn init(pg_pool: PgPool, config_path: PathBuf) -> Result<Self, Error> {
            let config = read_ethereum_config(config_path)?;
            let private_key = std::env::var(CASHIER_PRIVATE_KEY_ENV_VAR)
                .map_err(|e| Error::Env(CASHIER_PRIVATE_KEY_ENV_VAR, e))?;

            // Create provider
            let provider = Provider::<Http>::try_from(config.ethereum_rpc_node_url.as_str())
                .map_err(|_| Error::PrivateKey)?;

            // Create wallet
            let wallet = LocalWallet::from_str(&private_key)
                .map_err(|_| Error::PrivateKey)?
                .with_chain_id(config.chain_id.chain_id());

            // Create client with signer
            let client = Arc::new(SignerMiddleware::new(provider, wallet));

            let cloned_chain_id = config.chain_id.clone();
            let cloned_cashier_account_address = config.cashier_account_address;
            let cloned_pg_pool = pg_pool.clone();
            
            #[cfg(not(feature = "mock"))]
            {
                let _handle = tokio::spawn(async move {
                    indexer::init_indexer_task(
                        cloned_pg_pool,
                        config.ethereum_substreams_url,
                        cloned_chain_id,
                        cloned_cashier_account_address,
                    )
                    .await
                });
            }

            let on_chain_constants = ON_CHAIN_CONSTANTS.get(config.chain_id.as_str()).unwrap();

            // Create withdraw order channel
            let (withdraw_sender, withdraw_receiver) = mpsc::unbounded_channel();

            // Spawn withdraw processor (placeholder for now)
            let _client_clone = client.clone();
            let _invoice_contract_address = on_chain_constants.invoice_payment_contract_address;
            tokio::spawn(async move {
                // TODO: Implement Ethereum withdraw processing
                loop {
                    if let Some(_order) = withdraw_receiver.recv().await {
                        tracing::debug!("Received withdraw order for Ethereum processing");
                        // Process the order here
                    }
                }
            });

            Ok(EthereumLiquiditySource {
                depositer: Depositer::new(config.chain_id.clone(), config.cashier_account_address),
                withdrawer: Withdrawer::new(config.chain_id, withdraw_sender),
            })
        }
    }
}
