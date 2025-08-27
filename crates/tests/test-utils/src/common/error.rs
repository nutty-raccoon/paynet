use thiserror::Error;

#[cfg(feature = "concurrency")]
#[derive(Debug, Error)]
pub enum ConcurrencyError {
    Melt,
    Mint,
    Swap,
}

#[cfg(feature = "concurrency")]
impl core::fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ConcurrencyError::Melt => write!(f, "Melt"),
            ConcurrencyError::Mint => write!(f, "Mint"),
            ConcurrencyError::Swap => write!(f, "Swap"),
        }
    }
}

#[cfg(feature = "e2e")]
#[derive(Debug, Error)]
pub enum E2eError {
    Melt,
    Mint,
    Receive,
    Send,
}

#[cfg(feature = "e2e")]
impl core::fmt::Display for E2eError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            E2eError::Melt => write!(f, "Melt"),
            E2eError::Mint => write!(f, "Mint"),
            E2eError::Receive => write!(f, "Receive"),
            E2eError::Send => write!(f, "Send"),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[cfg(feature = "starknet")]
    #[error(transparent)]
    Provider(#[from] starknet::providers::ProviderError),
    #[error(transparent)]
    Grpc(#[from] tonic::Status),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[cfg(feature = "concurrency")]
    #[error(transparent)]
    Concurrence(#[from] ConcurrencyError),
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    E2e(#[from] E2eError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    R2d2(#[from] r2d2::Error),
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    Xpriv(#[from] bitcoin::bip32::Error),
    #[cfg(feature = "e2e")]
    #[error(transparent)]
    SeedPhrase(#[from] wallet::wallet::sqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
