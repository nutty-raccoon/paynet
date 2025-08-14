//! Unit handling for Ethereum tokens
//!
//! This module provides a type-safe representation of Ethereum units (wei, gwei, ether)
//! and their conversion to and from blockchain-native values like `U256`.

use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Asset;

/// String literals for supported Ethereum units
const GWEI_STR: &str = "gwei";

/// Represents Ethereum units used in the protocol
///
/// This abstraction allows user-facing amounts to be expressed in familiar units
/// while internally converting them to `U256`-based blockchain representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Gwei,
}

impl Unit {
    /// Maps each unit to the associated `Asset`
    pub fn asset(&self) -> Asset {
        Asset::Weth
    }

    /// Returns the string representation of the unit
    pub fn as_str(&self) -> &'static str {
        match self {
            Unit::Gwei => GWEI_STR,
        }
    }

    /// Returns the scale factor to convert unit into native base unit (`wei`)
    ///
    /// - 1 wei   = 1
    /// - 1 gwei  = 10^9 wei
    /// - 1 ether = 10^18 wei
    pub fn scale_factor(&self) -> u64 {
        match self {
            Unit::Gwei => 1_000_000_000,
        }
    }

    /// Returns the power of 10 for the scale factor
    pub fn scale_order(&self) -> u8 {
        match self {
            Unit::Gwei => 9,
        }
    }

    /// Converts a user-friendly `Amount` into blockchain-native `U256`
    pub fn convert_amount_into_u256(&self, amount: Amount) -> U256 {
        U256::from(u64::from(amount)) * U256::from(self.scale_factor())
    }

    /// Validates that the given asset is supported by this unit
    pub fn is_asset_supported(&self, asset: Asset) -> bool {
        matches!(asset, Asset::Weth)
    }
}

impl AsRef<str> for Unit {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Assigns a unique number for each unit for key derivation or indexing
impl From<Unit> for u32 {
    fn from(value: Unit) -> Self {
        match value {
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
        match s {
            GWEI_STR => Ok(Unit::Gwei),
            _ => Err(UnitFromStrError),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

/// Required trait implementation to work with `nuts::Amount`
impl nuts::traits::Unit for Unit {}
