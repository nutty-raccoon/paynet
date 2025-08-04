use serde::{Deserialize, Serialize};
use starknet::core::types::Call;
use starknet_crypto::poseidon_hash;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawOrder {
    pub quote_id_hash: Felt,
    pub expiry: Felt,
    pub asset_contract_address: Felt,
    pub amount: StarknetU256,
    pub payee: Felt,
}

impl WithdrawOrder {
    pub fn new(
        quote_id_hash: Felt,
        expiry: Felt,
        amount: StarknetU256,
        asset_contract_address: Felt,
        payee: Felt,
    ) -> Self {
        Self {
            quote_id_hash,
            expiry,
            asset_contract_address,
            amount,
            payee,
        }
    }

    pub fn to_starknet_calls(self, invoice_payment_contract_address: Felt) -> [Call; 2] {
        transactions::generate_single_payment_transaction_calls(
            invoice_payment_contract_address,
            self.quote_id_hash,
            self.expiry,
            self.asset_contract_address,
            &self.amount,
            self.payee,
        )
    }
}

pub type DepositPayload = WithdrawOrder;

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
pub fn compute_invoice_id<E: Into<Felt>>(quote_id_hash: Felt, expiry: E) -> Felt {
    // Convert expiry to Felt
    let expiry_felt = expiry.into();

    // Calculate poseidon hash
    poseidon_hash(quote_id_hash, expiry_felt)
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
