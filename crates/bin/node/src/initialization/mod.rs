mod commands;
pub use commands::ProgramArguments;
mod env_variables;
pub use env_variables::read_env_variables;
mod db;
mod nuts_settings;
pub use db::connect_to_db_and_run_migrations;
mod signer_client;
pub use signer_client::connect_to_signer;
mod grpc;
pub use grpc::launch_tonic_server_task;

use crate::grpc_service::InitKeysetError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to database: {0}")]
    DbConnect(#[source] sqlx::Error),
    #[error("Failed to run the database migration: {0}")]
    DbMigrate(#[source] sqlx::migrate::MigrateError),
    #[cfg(debug_assertions)]
    #[error("Failed to load .env file: {0}")]
    Dotenvy(#[source] dotenvy::Error),
    #[error("Failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Failed parse the Grpc address")]
    InvalidGrpcAddress(#[from] std::net::AddrParseError),
    #[error("failed to connect to signer")]
    SignerConnection(tonic::transport::Error),
    #[cfg(feature = "tls")]
    #[error("failed to setup gRPC server: {0}")]
    OpenSSL(#[from] openssl::error::ErrorStack),
    #[error("failed to bind gRPC server to port: {0}")]
    Bind(#[from] std::io::Error),
    #[error("failed to init first keysets: {0}")]
    InitKeysets(#[from] InitKeysetError),
}
