use commands::read_env_variables;
use errors::{Error, InitializationError, ServiceError};
use futures::TryFutureExt;
use grpc_service::GrpcState;
use methods::Method;
use node::KeysetRotationServiceServer;
use node::NodeServer;
use nuts::{
    Amount, QuoteTTLConfig, nut04::MintMethodSettings, nut05::MeltMethodSettings,
    nut06::NutsSettings,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use starknet_types::Unit;
use tokio::try_join;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod app_state;
mod commands;
mod errors;
mod grpc_service;
#[cfg(feature = "starknet")]
mod indexer;
mod keyset_cache;
mod keyset_rotation;
mod logic;
mod methods;
mod routes;
mod utils;

async fn connect_to_db_and_run_migrations(pg_url: &str) -> Result<PgPool, InitializationError> {
    let pool = PgPoolOptions::new()
        .max_connections(32)
        .min_connections(6)
        .connect(pg_url)
        .await
        .map_err(InitializationError::DbConnect)?;

    db_node::run_migrations(&pool)
        .await
        .map_err(InitializationError::DbMigrate)?;

    Ok(pool)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Initializing node...");

    // Read args and env
    #[cfg(feature = "starknet")]
    let config = {
        use clap::Parser;
        let args = commands::Args::parse();
        args.read_config()?
    };
    let env_variables = read_env_variables()?;

    // Connect to db
    let pg_pool = connect_to_db_and_run_migrations(&env_variables.pg_url).await?;

    // Define the node settings
    let nuts_settings = NutsSettings {
        nut04: nuts::nut04::Settings {
            methods: vec![MintMethodSettings {
                method: Method::Starknet,
                unit: Unit::Strk,
                min_amount: Some(Amount::ONE),
                max_amount: None,
                description: true,
            }],
            disabled: false,
        },
        nut05: nuts::nut05::Settings {
            methods: vec![MeltMethodSettings {
                method: Method::Starknet,
                unit: Unit::Strk,
                min_amount: Some(Amount::ONE),
                max_amount: None,
            }],
            disabled: false,
        },
    };

    // Connect to the signer service
    let signer_client = signer::SignerClient::connect(env_variables.signer_url)
        .await
        .map_err(InitializationError::SignerConnection)?;
    info!("Connected to signer server.");

    #[cfg(feature = "starknet")]
    let starknet_cashier = {
        let starknet_cashier =
            starknet_cashier::StarknetCashierClient::connect(env_variables.cashier_url)
                .await
                .map_err(InitializationError::SignerConnection)?;
        info!("Connected to starknet cashier server.");

        starknet_cashier
    };

    // Launch tonic server task
    let grpc_service = GrpcState::new(
        pg_pool.clone(),
        signer_client,
        nuts_settings,
        QuoteTTLConfig {
            mint_ttl: 3600,
            melt_ttl: 3600,
        },
        #[cfg(feature = "starknet")]
        starknet_cashier,
    );
    grpc_service
        .init_first_keysets(&[Unit::Strk], 0, 32)
        .await?;

    let addr = format!("[::0]:{}", env_variables.grpc_port)
        .parse()
        .map_err(|e| Error::Init(InitializationError::InvalidGrpcAddress(e)))?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<NodeServer<GrpcState>>().await;

    let tonic_future = tonic::transport::Server::builder()
        .add_service(NodeServer::new(grpc_service.clone()))
        .add_service(KeysetRotationServiceServer::new(grpc_service.clone()))
        .add_service(health_service)
        .serve(addr)
        .map_err(|e| Error::Service(ServiceError::TonicTransport(e)));

    // Launch indexer task
    #[cfg(feature = "starknet")]
    let indexer_future = {
        let indexer_service = indexer::init_indexer_task(
            env_variables.apibara_token,
            config.strk_address,
            config.recipient_address,
        )
        .await?;
        let db_conn = pg_pool.acquire().await?;
        let indexer_future = indexer::listen_to_indexer(db_conn, indexer_service);
        info!("Listening to starknet indexer!");

        indexer_future
    };

    info!("Initialized!");
    info!("Running gRPC server at {}", addr);
    println!("Running gRPC server at {}", addr);

    #[cfg(feature = "starknet")]
    let ((), ()) = try_join!(tonic_future, indexer_future)?;

    #[cfg(not(feature = "starknet"))]
    let ((),) = try_join!(tonic_future)?;

    Ok(())
}
