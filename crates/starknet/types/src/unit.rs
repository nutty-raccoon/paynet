//! Unit handling for Starknet tokens
//!
//! This module provides a type-safe representation of protocol's units and their conversion
//! to blockchain-native values.

use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{Asset, StarknetU256, StarknetU256ToAmountError};

/// Represents units supported by the node for user-facing operations
///
/// Units provide a domain-specific abstraction layer over raw blockchain assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    MilliStrk,
}

/// Maps a unit to its corresponding blockchain asset
///
/// This enables the application to maintain separate concepts for
/// user-facing units and blockchain assets while providing a clear
/// relationship between them.
impl Unit {
    pub fn asset(&self) -> Asset {
        match self {
            Unit::MilliStrk => Asset::Strk,
        }
    }
}

/// Used in the derivation path when createing keysets
///
/// This guarantee that different units don't share the same signing keys
impl From<Unit> for u32 {
    fn from(value: Unit) -> Self {
        match value {
            Unit::MilliStrk => 0,
        }
    }
}

/// Error returned when parsing an unknown unit string
#[derive(Debug, thiserror::Error)]
#[error("invalid value for enum `Unit`")]
pub struct UnitFromStrError;

impl FromStr for Unit {
    type Err = UnitFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit = match s {
            "strk" => Self::MilliStrk,
            _ => return Err(UnitFromStrError),
        };

        Ok(unit)
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            match self {
                Unit::MilliStrk => "strk",
            },
            f,
        )
    }
}

// Implementing nuts::traits::Unit enables this type to work with the rest of the protocol code.
// Required because we will be supporting different sets of Units in the future.
// Most likely, one by network we abstract.
impl nuts::traits::Unit for Unit {}

// Conversion factor between an `Unit::Strk` `Amount` and its blockchain-native representation.
// The starknet STRK token has a precision of 18. Meaning that 1 STRK = 10^18 wei.
// Because this protocol focus on real life payment, we represent user-facing amounts in milli-STRK (e-3 STRK),
// which is $0,0001786 at the time or writing those lines. I don't think we will ever need a smaller denomination.
// We could even arguee it's too small, but we really hope the token price will pump in the future.
//
// Therefore we need 10^15 as the conversion factor (10^18 / 10^3)
const MILLI_STRK_UNIT_TO_ASSET_CONVERSION_RATE: u64 = 1_000_000_000_000_000;

impl Unit {
    /// Converts an amount of unit to its blockchain-native representation
    pub fn convert_amount_into_u256(&self, amount: Amount) -> StarknetU256 {
        match self {
            // TODO: probably possible to optimize this operation
            Unit::MilliStrk => StarknetU256::from(
                U256::from(u64::from(amount))
                    * U256::from(MILLI_STRK_UNIT_TO_ASSET_CONVERSION_RATE),
            ),
        }
    }

    /// Converts a blockchain-native amount to a its `Amount` value
    ///
    /// Returns both the converted amount and any remainder that couldn't be represented
    /// in the user-friendly unit. This precision handling is critical for accurate accounting.
    pub fn convert_u256_into_amount(
        &self,
        amount: StarknetU256,
    ) -> Result<(Amount, StarknetU256), StarknetU256ToAmountError> {
        // TODO: add some unit tests for this impl
        match self {
            Unit::MilliStrk => {
                let (quotient, rem) = primitive_types::U256::from(&amount)
                    .div_mod(U256::from(MILLI_STRK_UNIT_TO_ASSET_CONVERSION_RATE));
                Ok((
                    Amount::from(
                        u64::try_from(quotient)
                            .map_err(|_| StarknetU256ToAmountError(*self, amount))?,
                    ),
                    StarknetU256::from(rem),
                ))
            }
        }
    }

    /// Verifies that an asset is compatible with this unit
    ///
    /// This check helps to catch accidental mismatches between units and assets early.
    pub fn is_asset_supported(&self, asset: Asset) -> bool {
        match (self, asset) {
            (Unit::MilliStrk, Asset::Strk) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    fn test_convert_u256_into_amount_helper(amount: u64) {
        let unit = Unit::MilliStrk;
        let amount_u256 = StarknetU256::from(U256::from(amount));
        let conversion_rate = U256::from(MILLI_STRK_UNIT_TO_ASSET_CONVERSION_RATE);

        let result_a = unit.convert_u256_into_amount(amount_u256);
        let (converted_amount, remainder) = result_a.unwrap();

        let expected_quotient = U256::from(amount) / conversion_rate;
        let expected_remainder = U256::from(amount) % conversion_rate;

        assert_eq!(u64::from(converted_amount), expected_quotient.as_u64());
        assert_eq!(U256::from(remainder), expected_remainder);

        let result_b = unit.convert_amount_into_u256(converted_amount);
        assert_eq!(
            U256::from(result_b) + expected_remainder,
            U256::from(amount)
        );
    }

    #[test]
    fn test_convert_u256_into_amount() {
        let values = vec![
            0,
            5000,
            U256::MAX.low_u64(),
            u128::MAX as u64,
            (u128::MAX as u64).wrapping_add(1),
            1e18 as u64,
            1e17 as u64,
            1e16 as u64,
        ];

        for value in values {
            test_convert_u256_into_amount_helper(value);
        }
    }
}
