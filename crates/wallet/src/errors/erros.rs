use thiserror::Error;
use tonic::Status;
use nuts::nut00::NutError;
use nuts::nut01::KeyError;
use nuts::nut02::KeysetIdError;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("gRPC error: {0}")]
    Grpc(#[from] Status),

    #[error("Nut error: {0}")]
    Nut(#[from] NutError),

    #[error("Key error: {0}")]
    Key(#[from] KeyError),

    #[error("Keyset ID error: {0}")]
    KeysetId(#[from] KeysetIdError),

    #[error("Amount overflow")]
    AmountOverflow,

    #[error("No matching keyset found")]
    NoMatchingKeyset,

    #[error("Proof not available")]
    ProofNotAvailable,

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
}

pub type Result<T> = std::result::Result<T, WalletError>;