use bitcoin::secp256k1;
use thiserror::Error;

mod keys;
pub use keys::*;
mod mint_keys;
pub use mint_keys::*;
mod public_key;
pub use public_key::*;
mod secret_key;
pub use secret_key::*;

/// Nut01 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Secp256k1 Error
    #[error(transparent)]
    Secp256k1(#[from] secp256k1::Error),
    /// Json Error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Invalid Pubkey size
    #[error("Invalid public key size: expected={expected}, found={found}")]
    InvalidPublicKeySize {
        /// Expected size
        expected: usize,
        /// Actual size
        found: usize,
    },
}
