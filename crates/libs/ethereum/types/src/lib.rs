// use serde::{Deserialize, Serialize};
mod assets;
pub use assets::*;
mod chain_id;
pub use chain_id::ChainId;
pub mod constants;
mod unit;
pub use unit::{Unit, UnitFromStrError};

pub const ETHEREUM_STR: &str = "ethereum";

/// Possible errors when parsing or constructing a `ChainId`.
#[derive(Debug, thiserror::Error)]
pub enum ChainIdParseError {
    /// The chain ID contains non-ASCII characters.
    #[error("NonAsciiCharacter")]
    NonAsciiCharacter,

    /// The chain ID exceeds 31 characters in length.
    #[error("StringTooLong")]
    StringTooLong,

    /// Failed to parse the chain ID from a hex or decimal string.
    #[error("InvalidFormat")]
    InvalidFormat,

    /// Error occurred while parsing an integer.
    #[error("IntParseError: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
}
