#[cfg(all(feature = "mock", feature = "starknet"))]
compile_error!("Only one of the features 'mock' and 'starknet' can be enabled at the same time");
#[cfg(not(any(feature = "mock", feature = "starknet")))]
compile_error!("At least one liquidity feature should be provided during compilation");

use errors::Error;
use initialization::{
    connect_to_db_and_run_migrations, connect_to_signer, launch_tonic_server_task,
    read_env_variables,
};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod app_state;
mod errors;
mod grpc_service;
mod initialization;
mod keyset_cache;
#[cfg(feature = "keyset-rotation")]
mod keyset_rotation;
mod liquidity_sources;
mod logic;
mod methods;
mod response_cache;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Initializing node...");
    let args = <initialization::ProgramArguments as clap::Parser>::parse();

    // Read args and env
    let env_variables = read_env_variables()?;

    // Connect to db
    let pg_pool = connect_to_db_and_run_migrations(&env_variables.pg_url).await?;
    info!("Connected to node database.");

    // Connect to the signer service
    let signer_client = connect_to_signer(env_variables.signer_url).await?;
    info!("Connected to signer server.");

    let liquidity_sources =
        liquidity_sources::LiquiditySources::init(pg_pool.clone(), args).await?;

    // Launch tonic server task
    let (address, grpc_future) = launch_tonic_server_task(
        pg_pool.clone(),
        signer_client,
        liquidity_sources,
        env_variables.grpc_port,
    )
    .await?;

    info!("Running gRPC server at {}", address);
    tokio::select! {
        grpc_res = grpc_future => match grpc_res {
            Ok(()) => eprintln!("gRPC task should never return"),
            Err(err) => eprintln!("gRPC task failed: {}", err),
        },
        sig = tokio::signal::ctrl_c() => match sig {
            Ok(()) => info!("gRPC task terminated"),
            Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
        }
    };

    Ok(())
}
