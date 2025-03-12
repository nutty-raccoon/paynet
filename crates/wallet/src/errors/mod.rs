use thiserror::Error;

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
    #[error("Invalid keyset ID: {0}")]
    InvalidKeysetId(String),
    #[error("gRPC error: {0}")]
    Grpc(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Nut01 error: {0}")]
    Nut01(String),
    #[error("Nut02 error: {0}")]
    Nut02(String),
    #[error("DHKE error: {0}")]
    Dhke(String),
}

pub type Result<T> = std::result::Result<T, WalletError>;

impl From<tonic::Status> for WalletError {
    fn from(value: tonic::Status) -> Self {
        WalletError::Grpc(value.message().to_string())
    }
}

impl From<nuts::Error> for WalletError {
    fn from(value: nuts::Error) -> Self {
        WalletError::Protocol(value.to_string())
    }
}

impl From<nuts::nut01::Error> for WalletError {
    fn from(value: nuts::nut01::Error) -> Self {
        WalletError::Nut01(value.to_string())
    }
}

impl From<nuts::nut02::Error> for WalletError {
    fn from(value: nuts::nut02::Error) -> Self {
        WalletError::Nut02(value.to_string())
    }
}

impl From<nuts::dhke::Error> for WalletError {
    fn from(value: nuts::dhke::Error) -> Self {
        WalletError::Dhke(value.to_string())
    }
}