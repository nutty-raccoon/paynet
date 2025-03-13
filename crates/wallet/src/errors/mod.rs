use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("Amount overflow")]
    AmountOverflow,
    #[error("No matching keyset found")]
    NoMatchingKeyset,
    #[error("Proof not available")]
    ProofNotAvailable,
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Invalid keyset ID")]
    InvalidKeysetId(#[from] std::array::TryFromSliceError),
    #[error("gRPC error: {0}")]
    Grpc(#[from] Status),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Nut01 error: {0}")]
    Nut01(#[from] nuts::nut01::Error),
    #[error("Nut02 error: {0}")]
    Nut02(#[from] nuts::nut02::Error),
    #[error("DHKE error: {0}")]
    Dhke(#[from] nuts::dhke::Error),
    #[error("Nuts error: {0}")]
    Nuts(#[from] nuts::Error),
}

pub type Result<T> = std::result::Result<T, WalletError>;