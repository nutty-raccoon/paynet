use axum::{Json, http::StatusCode, response::IntoResponse};
use nuts::{dhke, nut00::CashuError, nut02};
use thiserror::Error;

use crate::commands::ConfigError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Init(#[from] InitializationError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Dhke(#[from] dhke::Error),
    #[error(transparent)]
    Nut02(#[from] nut02::Error),
    #[error(transparent)]
    Database(#[from] db_node::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Starknet(#[from] starknet_types::Error),
    #[error(transparent)]
    Tonic(#[from] tonic::Status),
    #[error(transparent)]
    Service(#[from] ServiceError),
    #[error("Keyset doesn't exist in this mint")]
    UnknownKeySet,
    #[error("No keypair for amount")]
    InvalidAmountKey,
    #[error("A value overflowed during execution")]
    Overflow,
    #[error("The KeyManager generated a KeysetId different from the one known in db")]
    GeneratedKeysetIdIsDifferentFromOriginal,
    /// Inactive Keyset
    #[error("Inactive Keyset")]
    InactiveKeyset,
}

impl From<Error> for CashuError {
    fn from(_value: Error) -> Self {
        Self::new(0, String::new())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(CashuError::from(self))).into_response()
    }
}

#[derive(Debug, Error)]
pub enum InitializationError {
    #[error("Failed to read the config file: {0}")]
    CannotReadConfig(#[source] std::io::Error),
    #[error("Failed to deserialize the config file as toml: {0}")]
    Toml(#[source] toml::de::Error),
    #[error("Failed to connect to database: {0}")]
    DbConnect(#[source] sqlx::Error),
    #[error("Failed to run the database migration: {0}")]
    DbMigrate(#[source] sqlx::migrate::MigrateError),
    #[cfg(debug_assertions)]
    #[error("Failed to load .env file: {0}")]
    Dotenvy(#[source] dotenvy::Error),
    #[error("Failed to read variable from environment: {0}")]
    Env(#[source] std::env::VarError),
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("Failed init apibara indexer: {0}")]
    InitIndexer(#[source] starknet_payment_indexer::Error),
    #[error("Failed bind tcp listener: {0}")]
    BindTcp(#[source] std::io::Error),
    #[error("Failed to open the SqLite db: {0}")]
    OpenSqlite(#[source] rusqlite::Error),
    #[error("Failed parse the Grpc address")]
    InvalidGrpcAddress,
    #[error("failed to connect to signer")]
    SignerConnection(tonic::transport::Error),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Failed to run the indexer: {0}")]
    Indexer(#[source] anyhow::Error),
    #[error("Failed to serve the axum server: {0}")]
    AxumServe(#[source] std::io::Error),
    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
}
