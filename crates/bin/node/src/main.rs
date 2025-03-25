use clap::Parser;
use errors::Error;
use initialization::{
    ProgramArguments, connect_to_db_and_run_migrations, connect_to_signer, launch_indexer_task,
    launch_tonic_server_task, read_env_variables,
};
use node::KeysetRotationServiceServer;
use starknet_types::Unit;
use tokio::try_join;
use tonic_health::pb::health_server::HealthServer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod app_state;
mod errors;
mod grpc_service;
#[cfg(feature = "starknet")]
mod indexer;
mod initialization;
mod keyset_cache;
mod keyset_rotation;
mod logic;
mod methods;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Initializing node...");
    #[cfg(feature = "starknet")]
    let args = ProgramArguments::parse();
    #[cfg(feature = "starknet")]
    let starknet_config = args.read_starknet_config()?;

    // Read args and env
    #[cfg(feature = "starknet")]
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
        let indexer_future =
            launch_indexer_task(&pg_pool, env_variables.apibara_token, &starknet_config).await?;
        info!("Listening to starknet indexer.");

        indexer_future
    };

    // Launch tonic server task
    let (address, grpc_future, health_reporter) = launch_tonic_server_task(
        pg_pool.clone(),
        signer_client,
        #[cfg(feature = "starknet")]
        starknet_cashier,
        env_variables.grpc_port,
    )
    .await?;
    info!("Running gRPC server at {}", address);

    // We are done initializing
    info!("Initialized!");

    // Run them forever
    #[cfg(feature = "starknet")]
    let ((), ()) = try_join!(grpc_future, indexer_future, set_serving(health_reporter))?;
    // or
    #[cfg(not(feature = "starknet"))]
    let ((),) = try_join!(tonic_future, set_serving(health_reporter))?;

    Ok(())
}
