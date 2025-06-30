use core::fmt;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConcurrenceError {
    Melt,
    Mint,
    Swap,
}

impl fmt::Display for ConcurrenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConcurrenceError::Melt => write!(f, "Melt"),
            ConcurrenceError::Mint => write!(f, "Mint"),
            ConcurrenceError::Swap => write!(f, "Swap"),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Provider(#[from] starknet::providers::ProviderError),
    #[error(transparent)]
    Grpc(#[from] tonic::Status),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[error(transparent)]
    Concurrence(#[from] ConcurrenceError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
