use num_bigint::BigUint;
use nuts::Amount;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

mod unit;
pub use unit::{Unit, UnitFromStrError};
mod method;
pub use method::{Method, MethodFromStrError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Starknet u256 amount of {1} is to big to be converted into a cashu Amount for unit {0}"
    )]
    StarknetAmountTooHigh(Unit, StarknetU256),
}

pub const STRK_TOKEN_ADDRESS: Felt =
    Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d");

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    Strk,
}

impl core::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Asset::Strk => "strk",
            }
        )
    }
}

impl Asset {
    pub fn address(&self) -> Felt {
        STRK_TOKEN_ADDRESS
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeltPaymentRequest {
    pub recipient: Felt,
    pub asset: Asset,
    pub amount: StarknetU256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintPaymentRequest<C> {
    pub contract_address: Felt,
    pub selector: Felt,
    pub calldata: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayInvoiceCalldata {
    pub invoice_id: u128,
    pub asset: Asset,
    pub amount: StarknetU256,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarknetU256 {
    pub low: Felt,
    pub high: Felt,
}

impl StarknetU256 {
    pub const ZERO: StarknetU256 = StarknetU256 {
        low: Felt::ZERO,
        high: Felt::ZERO,
    };
}

impl StarknetU256 {
    pub fn from_parts<L: Into<u128>, H: Into<u128>>(low: L, high: H) -> Self {
        let low: u128 = low.into();
        let high: u128 = high.into();
        Self {
            low: Felt::from(low),
            high: Felt::from(high),
        }
    }
}

impl core::fmt::Display for StarknetU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "low: {:#x} - high: {:#x}", self.low, self.high)
    }
}

impl From<Amount> for StarknetU256 {
    fn from(value: Amount) -> Self {
        Self {
            low: value.into(),
            high: Felt::ZERO,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TryU256FromBigUintError {
    #[error("BigUint too big")]
    TooBig,
}

impl TryFrom<BigUint> for StarknetU256 {
    type Error = TryU256FromBigUintError;

    fn try_from(value: BigUint) -> Result<Self, Self::Error> {
        let bytes = value.to_bytes_le();
        if bytes.len() > 32 {
            return Err(Self::Error::TooBig);
        };

        if bytes.len() < 16 {
            return Ok(StarknetU256 {
                low: Felt::from_bytes_le_slice(&bytes),
                high: Felt::ZERO,
            });
        }

        Ok(StarknetU256 {
            low: Felt::from_bytes_le_slice(&bytes[0..16]),
            high: Felt::from_bytes_le_slice(&bytes[16..]),
        })
    }
}

impl From<primitive_types::U256> for StarknetU256 {
    fn from(value: primitive_types::U256) -> Self {
        let bytes = value.to_little_endian();
        let low = u128::from_le_bytes(bytes[..16].try_into().unwrap());
        let high = u128::from_le_bytes(bytes[16..].try_into().unwrap());
        Self {
            low: Felt::from(low),
            high: Felt::from(high),
        }
    }
}

impl From<StarknetU256> for primitive_types::U256 {
    fn from(value: StarknetU256) -> Self {
        Self::from(&value)
    }
}

impl From<&StarknetU256> for primitive_types::U256 {
    fn from(value: &StarknetU256) -> Self {
        let mut bytes = value.low.to_bytes_le();
        bytes[16..].copy_from_slice(&value.high.to_bytes_le()[..16]);

        primitive_types::U256::from_little_endian(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use starknet_types_core::felt::Felt;

    use crate::StarknetU256;

    #[test]
    fn starknet_and_primitive_types_u256_conversion() {
        let pt = primitive_types::U256::MAX;
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let pt = primitive_types::U256::zero();
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let pt = primitive_types::U256::one();
        let s = StarknetU256::from(pt);

        assert_eq!(primitive_types::U256::from(s), pt);

        let s = StarknetU256 {
            low: Felt::from_hex_unchecked("0xbabe"),
            high: Felt::from_hex_unchecked("0xcafe"),
        };
        let pt = primitive_types::U256::from(&s);

        assert_eq!(StarknetU256::from(pt), s);
    }
}

pub fn felt_to_short_string(felt: Felt) -> Result<String, std::string::FromUtf8Error> {
    let bytes = felt.to_bytes_be();

    String::from_utf8(bytes.to_vec())
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
#[cfg(test)]
mod tests {
    use super::*;
    use primitive_types::U256;
    use num_bigint::BigUint;
    use std::str::FromStr;

    // Test conversion from Amount to StarknetU256
    #[test]
    fn test_amount_to_starknet_u256_conversion() {
        let amount = Amount::from(42u128);
        let starknet_u256 = StarknetU256::from(amount);
        
        assert_eq!(starknet_u256.low, Felt::from(42u128));
        assert_eq!(starknet_u256.high, Felt::ZERO);
    }

    // Test conversion from BigUint to StarknetU256
    #[test]
    fn test_biguint_to_starknet_u256_conversion() {
        // Test small number
        let small_biguint = BigUint::from_str("12345").unwrap();
        let small_u256 = StarknetU256::try_from(small_biguint.clone()).unwrap();
        assert_eq!(small_u256.low, Felt::from(12345u128));
        assert_eq!(small_u256.high, Felt::ZERO);

        // Test larger number spanning both low and high
        let large_biguint = BigUint::from_str("340282366920938463463374607431768211456").unwrap(); // 2^128
        let large_u256 = StarknetU256::try_from(large_biguint.clone()).unwrap();
        assert_eq!(large_u256.low, Felt::ZERO);
        assert_eq!(large_u256.high, Felt::from(1u128));
    }

    // Test BigUint conversion failure for numbers too large
    #[test]
    fn test_biguint_to_starknet_u256_conversion_failure() {
        let too_large_biguint = BigUint::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639936").unwrap(); // 2^256
        let result = StarknetU256::try_from(too_large_biguint);
        assert!(result.is_err());
    }

    // Test felt_to_short_string function
    #[test]
    fn test_felt_to_short_string() {
        // Test ASCII conversion
        let felt = felt_from_short_string("hello").unwrap();
        let result = felt_to_short_string(felt).unwrap();
        assert_eq!(result, "hello");
    }

    // Test felt_from_short_string validation
    #[test]
    fn test_felt_from_short_string_validation() {
        // Test ASCII string
        assert!(felt_from_short_string("valid").is_ok());

        // Test non-ASCII string
        assert!(felt_from_short_string("こんにちは").is_err());

        // Test string too long
        assert!(felt_from_short_string("this_string_is_definitely_longer_than_thirty_one_characters").is_err());
    }

    // Test Asset enum functionality
    #[test]
    fn test_asset_enum() {
        let strk_asset = Asset::Strk;
        
        // Test Display trait
        assert_eq!(strk_asset.to_string(), "strk");
        
        // Test address
        assert_eq!(strk_asset.address(), STRK_TOKEN_ADDRESS);
    }

    // Test StarknetU256 static methods
    #[test]
    fn test_starknet_u256_methods() {
        // Test ZERO constant
        assert_eq!(StarknetU256::ZERO.low, Felt::ZERO);
        assert_eq!(StarknetU256::ZERO.high, Felt::ZERO);

        // Test from_parts method
        let u256 = StarknetU256::from_parts(42u128, 24u128);
        assert_eq!(u256.low, Felt::from(42u128));
        assert_eq!(u256.high, Felt::from(24u128));
    }
}
