use starknet_types::ChainId;

mod commands;
#[cfg(feature = "starknet")]
pub use commands::ProgramArguments;
#[cfg(feature = "starknet")]
pub use commands::StarknetConfig;
mod env_variables;
pub use env_variables::read_env_variables;
mod db;
mod nuts_settings;
pub use db::connect_to_db_and_run_migrations;
mod signer_client;
pub use signer_client::connect_to_signer;
#[cfg(feature = "starknet")]
mod starknet_cashier_client;
#[cfg(feature = "starknet")]
pub use starknet_cashier_client::connect_to_starknet_cashier;
mod grpc;
pub use grpc::launch_tonic_server_task;
#[cfg(feature = "starknet")]
mod indexer;
#[cfg(feature = "starknet")]
pub use indexer::launch_indexer_task;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    #[error("Failed to read environment variable `{0}`: {1}")]
    Env(&'static str, #[source] std::env::VarError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[cfg(feature = "starknet")]
    #[error("Failed init apibara indexer: {0}")]
    InitIndexer(#[source] starknet_payment_indexer::Error),
    #[error("Failed bind tcp listener: {0}")]
    BindTcp(#[source] std::io::Error),
    #[error("Failed to open the SqLite db: {0}")]
    OpenSqlite(#[source] rusqlite::Error),
    #[error("Failed parse the Grpc address")]
    InvalidGrpcAddress(#[from] std::net::AddrParseError),
    #[error("unknown chain id: {0}")]
    UnknownChainId(ChainId),
    #[error("failed to connect to signer")]
    SignerConnection(tonic::transport::Error),
    #[cfg(feature = "starknet")]
    #[error(transparent)]
    Cashier(#[from] starknet_cashier_client::Error),
}
