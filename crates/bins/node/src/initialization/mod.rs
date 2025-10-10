mod env_variables;
pub use env_variables::read_env_variables;
mod db;
mod nuts_settings;
pub use db::connect_to_db_and_run_migrations;
mod signer_client;
pub use signer_client::connect_to_signer;
mod grpc;
pub use grpc::launch_tonic_server_task;
#[cfg(feature = "rest")]
mod rest;
#[cfg(feature = "rest")]
pub use rest::launch_rest_server_task;
use tracing::instrument;

use crate::grpc_service::InitKeysetError;
use crate::{app_state::AppState, liquidity_sources::LiquiditySources};
use nuts::QuoteTTLConfig;
use nuts_settings::nuts_settings;
use signer::SignerClient;
use sqlx::Postgres;
use starknet_types::Unit;
use tonic::transport::Channel;
use tower_otel::trace;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to database: {0}")]
    DbConnect(#[source] sqlx::Error),
    #[error("Failed to run the database migration: {0}")]
    DbMigrate(#[source] sqlx::migrate::MigrateError),
    #[error("Failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Failed parse the Grpc address")]
    InvalidGrpcAddress(#[from] std::net::AddrParseError),
    #[error("failed to connect to signer")]
    SignerConnection(tonic::transport::Error),
    #[error("failed to bind gRPC server to port: {0}")]
    Bind(#[from] std::io::Error),
    #[error("failed to init first keysets: {0}")]
    InitKeysets(#[from] InitKeysetError),
    #[error("invalid signer uri: {0}")]
    Uri(#[from] http::uri::InvalidUri),
    #[error("failed to build tonic server: {0}")]
    BuildServer(#[source] anyhow::Error),
    #[error("failed to build tonic reflection service: {0}")]
    TonicReflexion(#[from] tonic_reflection::server::Error),
}

#[instrument]
pub async fn create_app_state(
    pg_pool: sqlx::Pool<Postgres>,
    signer_client: SignerClient<trace::Grpc<Channel>>,
    liquidity_sources: LiquiditySources<Unit>,
    quote_ttl: Option<u64>,
) -> Result<AppState, super::Error> {
    let nuts_settings = nuts_settings();
    let ttl = quote_ttl.unwrap_or(3600);
    let app_state = AppState::new(
        pg_pool,
        signer_client,
        nuts_settings,
        QuoteTTLConfig {
            mint_ttl: ttl,
            melt_ttl: ttl,
        },
        liquidity_sources,
    );

    // Initialize first keysets
    app_state
        .init_first_keysets(&[Unit::MilliStrk], 0, 32)
        .await
        .map_err(|e| Error::InitKeysets(e))?;

    Ok(app_state)
}
