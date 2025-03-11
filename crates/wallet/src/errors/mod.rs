use thiserror::Error;
use tonic::transport;
use nuts::error::Error as NutsError;



#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("gRPC error: {0}")]
    Grpc(#[from] Status),

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