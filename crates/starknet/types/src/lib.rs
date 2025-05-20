use serde::{Deserialize, Serialize};
use starknet_crypto::PoseidonHasher;
use starknet_types_core::felt::Felt;

mod assets;
pub use assets::*;
mod u256;
pub use u256::*;
mod unit;
pub use unit::{Unit, UnitFromStrError};
mod chain_id;
pub mod constants;
pub use chain_id::ChainId;
pub mod transactions;

pub const STARKNET_STR: &str = "starknet";

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

/// Validates that a Felt value represents a valid Starknet contract address.
///
/// In Starknet, contract addresses must follow specific constraints to be considered valid:
/// - They must be greater than or equal to 2, as addresses 0 and 1 are reserved for system use:
///   * 0x0 acts as the default caller address for external calls and has no storage
///   * 0x1 functions as a storage space for block mapping [link](https://docs.starknet.io/architecture-and-concepts/network-architecture/starknet-state/#special_addresses)
/// - They must be less than 2^251 (0x800000000000000000000000000000000000000000000000000000000000000)
///
/// This validation is critical for preventing funds from being sent to invalid addresses,
/// which would result in permanent loss.
pub fn is_valid_starknet_address(felt: &Felt) -> bool {
    felt >= &Felt::from(2u64)
        && felt
            < &Felt::from_hex_unchecked(
                "0x800000000000000000000000000000000000000000000000000000000000000",
            )
}

/// Calculate the invoice_id using poseidon hash of quote_id_hash and expiry
pub fn calculate_invoice_id(quote_id_hash: &Felt, expiry: u64) -> Felt {
    // Convert expiry to Felt
    let expiry_felt = Felt::from(expiry);

    // Calculate poseidon hash
    let mut hash_state = PoseidonHasher::new();
    hash_state.update(*quote_id_hash);
    hash_state.update(expiry_felt);
    hash_state.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starknet_address_validation() {
        let valid_address1 = Felt::from_hex_unchecked(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        );
        let valid_address2 = Felt::from(100u64);
        let valid_address3 = Felt::from(2u64);

        // Invalid addresses
        let invalid_address1 = Felt::from(0u64);
        let invalid_address2 = Felt::from(1u64);
        let invalid_address4 = Felt::from_hex_unchecked(
            "0x800000000000000000000000000000000000000000000000000000000000000",
        );
        let invalid_address5 = Felt::from_hex_unchecked(
            "0x800000000000000000000000000000000000000000000000000000000000001",
        );

        assert!(is_valid_starknet_address(&valid_address1));
        assert!(is_valid_starknet_address(&valid_address2));
        assert!(is_valid_starknet_address(&valid_address3));

        assert!(!is_valid_starknet_address(&invalid_address1));
        assert!(!is_valid_starknet_address(&invalid_address2));
        assert!(!is_valid_starknet_address(&invalid_address4));
        assert!(!is_valid_starknet_address(&invalid_address5));
    }
}
