#[cfg(all(feature = "mock", feature = "starknet"))]
compile_error!("Only one of the features 'mock' and 'starknet' can be enabled at the same time");

use errors::Error;
use initialization::{
    connect_to_db_and_run_migrations, connect_to_signer, launch_tonic_server_task,
    read_env_variables,
};
use tokio::task::JoinSet;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[cfg_attr(not(any(feature = "mock", feature = "starknet")), allow(dead_code))]
mod app_state;
mod errors;
#[cfg_attr(
    not(any(feature = "mock", feature = "starknet")),
    allow(dead_code, unused_variables, unused_imports)
)]
mod grpc_service;
#[cfg(feature = "starknet")]
mod indexer;
mod initialization;
mod keyset_cache;
mod keyset_rotation;
mod logic;
mod methods;
mod routes;
#[cfg(any(feature = "mock", feature = "starknet"))]
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Initializing node...");
    #[cfg(feature = "starknet")]
    let args = <initialization::ProgramArguments as clap::Parser>::parse();
    #[cfg(feature = "starknet")]
    let starknet_config = args.read_starknet_config()?;

    // Read args and env
    let env_variables = read_env_variables()?;

    // Connect to db
    let pg_pool = connect_to_db_and_run_migrations(&env_variables.pg_url).await?;
    info!("Connected to node database.");

    // Connect to the signer service
    let signer_client = connect_to_signer(env_variables.signer_url).await?;
    info!("Connected to signer server.");

    #[cfg(feature = "starknet")]
    let starknet_cashier = {
        let starknet_cashier = initialization::connect_to_starknet_cashier(
            env_variables.cashier_url,
            starknet_config.chain_id.clone(),
        )
        .await?;
        info!("Connected to starknet cashier server.");

        starknet_cashier
    };

    // Launch indexer task
    #[cfg(feature = "starknet")]
    let indexer_future = {
        let indexer_future = initialization::launch_indexer_task(
            pg_pool.acquire().await?,
            env_variables.apibara_token,
            starknet_config.clone(),
        )
        .await?;
        info!("Listening to starknet indexer.");

        indexer_future
    };

    // Launch tonic server task
    let (address, grpc_future) = launch_tonic_server_task(
        pg_pool.clone(),
        signer_client,
        #[cfg(feature = "starknet")]
        app_state::starknet::StarknetConfig {
            withdrawer: liquidity_source::starknet::StarknetWithdrawer::new(starknet_cashier),
            depositer: liquidity_source::starknet::StarknetDepositer::new(
                starknet_config.chain_id,
                starknet_config.our_account_address,
            ),
        },
        env_variables.grpc_port,
    )
    .await?;
    info!("Running gRPC server at {}", address);

    // We are done initializing
    info!("Initialized!");

    let mut set = JoinSet::new();
    set.spawn(grpc_future);
    #[cfg(feature = "starknet")]
    set.spawn(indexer_future);

    // Run them forever
    let _ = set.join_all().await;

    Ok(())
}
