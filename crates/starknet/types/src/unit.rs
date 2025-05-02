//! Unit handling for Starknet tokens
//!
//! This module provides a type-safe representation of protocol's units and their conversion
//! to blockchain-native values.

use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Asset;

const GWEI_STR: &str = "gwei";
const MILLI_STR: &str = "millistrk";

/// Represents units supported by the node for user-facing operations
///
/// Units provide a domain-specific abstraction layer over raw blockchain assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    MilliStrk,
    Gwei,
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
            Unit::Gwei => Asset::Eth,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Unit::MilliStrk => MILLI_STR,
            Unit::Gwei => GWEI_STR,
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
            Unit::Gwei => 1,
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
            MILLI_STR => Self::MilliStrk,
            GWEI_STR => Self::Gwei,
            _ => return Err(UnitFromStrError),
        };

        Ok(unit)
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            match self {
                Unit::MilliStrk => MILLI_STR,
                Unit::Gwei => GWEI_STR,
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
    pub fn conversion_rate(&self) -> u64 {
        match self {
            Unit::MilliStrk => MILLI_STRK_UNIT_TO_ASSET_CONVERSION_RATE,
            Unit::Gwei => 1000000000,
        }
    }

    /// Converts an amount of unit to its blockchain-native representation
    pub fn convert_amount_into_u256(&self, amount: Amount) -> U256 {
        U256::from(u64::from(amount)) * U256::from(self.conversion_rate())
    }
    ///
    /// Verifies that an asset is compatible with this unit
    ///
    /// This check helps to catch accidental mismatches between units and assets early.
    pub fn is_asset_supported(&self, asset: Asset) -> bool {
        matches!(
            (self, asset),
            (Unit::MilliStrk, Asset::Strk) | (Unit::Gwei, Asset::Eth)
        )
    }
}
