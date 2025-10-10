#[cfg(not(any(feature = "grpc", feature = "rest")))]
compile_error!("You must enable at least one feature: `grpc` or `rest`.");
#[cfg(not(any(feature = "starknet")))]
compile_error!("At least one liquidity feature should be provided during compilation");

use core::panic;
use std::time::Duration;

use errors::Error;
use gauge::DbMetricsObserver;
#[cfg(feature = "rest")]
use initialization::launch_rest_server_task;
use initialization::{
    connect_to_db_and_run_migrations, connect_to_signer, launch_tonic_server_task,
    read_env_variables,
};
use tracing::{info, trace};

use crate::initialization::create_app_state;

mod app_state;
mod errors;
mod gauge;
mod grpc_service;
mod initialization;
mod keyset_cache;
#[cfg(feature = "keyset-rotation")]
mod keyset_rotation;
mod liquidity_sources;
mod logic;
mod methods;
mod response_cache;
#[cfg(feature = "rest")]
mod rest_service;
mod routes;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    let (meter_provider, subscriber) = open_telemetry_tracing::init(PKG_NAME, PKG_VERSION);

    tracing::subscriber::set_global_default(subscriber).unwrap();
    opentelemetry::global::set_meter_provider(meter_provider);

    info!("Initializing node...");
    // Read args and env
    let env_variables = read_env_variables()?;

    // Connect to db
    let pg_pool = connect_to_db_and_run_migrations(&env_variables.pg_url).await?;
    info!("Connected to node database.");

    // Lauch the database metrics polling task
    let meter = opentelemetry::global::meter("business");
    let gauge = meter.u64_gauge("stock").build();
    let observer = DbMetricsObserver::new(
        pg_pool.clone(),
        vec![starknet_types::Unit::MilliStrk],
        gauge,
    );
    let _handle = tokio::spawn(gauge::run_metrics_polling(
        observer,
        Duration::from_secs(60),
    ));

    // Connect to the signer service
    let signer_client = connect_to_signer(env_variables.signer_url.clone()).await?;
    info!("Connected to signer server.");

    let liquidity_sources = liquidity_sources::LiquiditySources::init(pg_pool.clone()).await?;

    // Create shared AppState
    let app_state = create_app_state(
        pg_pool.clone(),
        signer_client,
        liquidity_sources,
        env_variables.quote_ttl,
    )
    .await?;
    #[cfg(all(feature = "grpc", not(feature = "rest")))]
    {
        // Only gRPC enabled
        info!("Starting gRPC server only");
        let (grpc_address, grpc_future) =
            launch_tonic_server_task(app_state, env_variables.grpc_port).await?;
        trace!(name: "grpc-listen", port = grpc_address.port());

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
    }

    #[cfg(all(feature = "rest", not(feature = "grpc")))]
    {
        info!("Starting REST server only");
        let rest_future = launch_rest_server_task(app_state, env_variables.rest_port);

        tokio::select! {
            http_res = rest_future => match http_res {
                Ok(()) => eprintln!("REST task should never return"),
                Err(err) => eprintln!("REST task failed: {}", err),
            },
            sig = tokio::signal::ctrl_c() => match sig {
                Ok(()) => info!("REST task terminated"),
                Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
            }
        };
    }

    #[cfg(all(feature = "grpc", feature = "rest"))]
    {
        // Both gRPC and REST enabled
        info!("Starting both gRPC and REST servers");
        let (grpc_address, grpc_future) =
            launch_tonic_server_task(app_state.clone(), env_variables.grpc_port).await?;
        trace!(name: "grpc-listen", port = grpc_address.port());

        let rest_future = launch_rest_server_task(app_state, env_variables.rest_port);

        tokio::select! {
            grpc_res = grpc_future => match grpc_res {
                Ok(()) => eprintln!("gRPC task should never return"),
                Err(err) => eprintln!("gRPC task failed: {}", err),
            },
            http_res = rest_future => match http_res {
                Ok(()) => eprintln!("REST task should never return"),
                Err(err) => eprintln!("REST task failed: {}", err),
            },
            sig = tokio::signal::ctrl_c() => match sig {
                Ok(()) => info!("Servers terminated"),
                Err(err) => eprintln!("unable to listen for shutdown signal: {}", err)
            }
        };
    }

    Ok(())
}
