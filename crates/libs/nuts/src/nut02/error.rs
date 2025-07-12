use std::array::TryFromSliceError;

use thiserror::Error;

/// NUT02 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Hex Error
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),
    /// Keyset length error
    #[error("NUT02: ID length invalid")]
    Length,
    /// Unknown version
    #[error("NUT02: Unknown Version")]
    UnknownVersion,
    /// Keyset id does not match
    #[error("Keyset id incorrect")]
    IncorrectKeysetId,
    /// Slice Error
    #[error(transparent)]
    Slice(#[from] TryFromSliceError),
}
