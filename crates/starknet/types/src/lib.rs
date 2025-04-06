use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

mod assets;
pub use assets::*;
mod u256;
pub use u256::*;
mod unit;
pub use unit::{Unit, UnitFromStrError};
mod method;
pub use method::{Method, MethodFromStrError};
mod chain_id;
pub mod constants;
pub use chain_id::ChainId;
pub mod transactions;

#[derive(Debug, thiserror::Error)]
#[error("starknet u256 amount of {1} is to big to be converted into protocol Amount of Unit {0}")]
pub struct StarknetU256ToAmountError(Unit, StarknetU256);

pub fn felt_to_short_string(felt: Felt) -> String {
    let bytes = felt.to_bytes_be();
    let first_char_idx = match bytes.iter().position(|&b| b != 0) {
        Some(idx) => idx,
        None => return String::new(),
    };

    unsafe { String::from_utf8_unchecked(bytes[first_char_idx..].to_vec()) }
}

/// Possible errors for encoding a Cairo short string.
#[derive(Debug, thiserror::Error)]
pub enum CairoShortStringToFeltError {
    /// The string provided contains non-ASCII characters.
    #[error("NonAsciiCharacter")]
    NonAsciiCharacter,
    /// The string provided is longer than 31 characters.
    #[error("StringTooLong")]
    StringTooLong,
}

pub fn felt_from_short_string(s: &str) -> Result<Felt, CairoShortStringToFeltError> {
    if !s.is_ascii() {
        return Err(CairoShortStringToFeltError::NonAsciiCharacter);
    }
    if s.len() > 31 {
        return Err(CairoShortStringToFeltError::StringTooLong);
    }

    let ascii_bytes = s.as_bytes();

    let mut buffer = [0u8; 32];
    buffer[(32 - ascii_bytes.len())..].copy_from_slice(ascii_bytes);

    // The conversion will never fail
    Ok(Felt::from_bytes_be(&buffer))
}

// TODO: remove and use starknet-core struct when https://github.com/xJonathanLEI/starknet-rs/pull/713 is merged
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Call {
    /// Address of the contract being invoked.
    pub to: Felt,
    /// Entrypoint selector of the function being invoked.
    pub selector: Felt,
    /// List of calldata to be sent for the call.
    pub calldata: Vec<Felt>,
}

impl From<starknet::core::types::Call> for Call {
    fn from(value: starknet::core::types::Call) -> Self {
        Self {
            to: value.to,
            selector: value.selector,
            calldata: value.calldata,
        }
    }
}
impl From<Call> for starknet::core::types::Call {
    fn from(value: Call) -> Self {
        Self {
            to: value.to,
            selector: value.selector,
            calldata: value.calldata,
        }
    }
}
