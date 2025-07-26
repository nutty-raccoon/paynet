//! Unit handling for Ethereum tokens
//!
//! This module provides a type-safe representation of protocol's units and their conversion
//! to blockchain-native values.

use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Asset;

const GWEI_STR: &str = "gwei";
const MILLIUSDC_STR: &str = "milliusdc";

/// Represents units supported by the node for user-facing operations
///
/// Units provide a domain-specific abstraction layer over raw blockchain assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    MilliUsdc,
    Gwei,
}

impl Unit {
    /// Returns the underlying asset for this unit
    pub fn asset(&self) -> Asset {
        match self {
            Unit::MilliUsdc => Asset::Usdc,
            Unit::Gwei => Asset::Eth,
        }
    }

    /// Converts a protocol amount to the blockchain's native representation
    ///
    /// This handles the conversion from user-facing units to the actual values
    /// that will be used in blockchain transactions.
    pub fn convert_amount_into_u256(&self, amount: Amount) -> U256 {
        let amount_u64: u64 = amount.into();
        match self {
            Unit::MilliUsdc => {
                // 1 milliusdc = 0.001 USDC = 1000 micro-USDC
                // USDC has 6 decimals, so 1 USDC = 10^6 micro-USDC
                // 1 milliusdc = 1000 micro-USDC = 1000 * 10^(-6) USDC = 10^3 * 10^(-6) USDC = 10^(-3) USDC
                // In micro-USDC: 1 milliusdc = 1000 micro-USDC
                U256::from(amount_u64) * U256::from(1000)
            }
            Unit::Gwei => {
                // 1 gwei = 10^9 wei
                U256::from(amount_u64) * U256::from(10).pow(9.into())
            }
        }
    }

    /// Converts a blockchain amount back to protocol units
    ///
    /// This is the inverse of `convert_amount_into_u256`.
    pub fn convert_u256_into_amount(&self, blockchain_amount: U256) -> Result<Amount, ConversionError> {
        match self {
            Unit::MilliUsdc => {
                let divisor = U256::from(1000);
                if blockchain_amount % divisor != U256::zero() {
                    return Err(ConversionError::PrecisionLoss);
                }
                let amount = blockchain_amount / divisor;
                if amount > U256::from(u64::MAX) {
                    return Err(ConversionError::Overflow);
                }
                Ok(Amount::from(amount.as_u64()))
            }
            Unit::Gwei => {
                let divisor = U256::from(10).pow(9.into());
                if blockchain_amount % divisor != U256::zero() {
                    return Err(ConversionError::PrecisionLoss);
                }
                let amount = blockchain_amount / divisor;
                if amount > U256::from(u64::MAX) {
                    return Err(ConversionError::Overflow);
                }
                Ok(Amount::from(amount.as_u64()))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Conversion would result in precision loss")]
    PrecisionLoss,
    #[error("Amount too large to represent")]
    Overflow,
}

impl AsRef<str> for Unit {
    fn as_ref(&self) -> &str {
        match self {
            Unit::MilliUsdc => MILLIUSDC_STR,
            Unit::Gwei => GWEI_STR,
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnitFromStrError {
    #[error("Unknown unit: {0}")]
    UnknownUnit(String),
}

impl FromStr for Unit {
    type Err = UnitFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            GWEI_STR => Ok(Unit::Gwei),
            MILLIUSDC_STR => Ok(Unit::MilliUsdc),
            _ => Err(UnitFromStrError::UnknownUnit(s.to_string())),
        }
    }
}

impl From<Unit> for u32 {
    fn from(unit: Unit) -> Self {
        match unit {
            Unit::MilliUsdc => 0,
            Unit::Gwei => 1,
        }
    }
}

impl nuts::traits::Unit for Unit {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_from_str() {
        assert_eq!(Unit::from_str("gwei").unwrap(), Unit::Gwei);
        assert_eq!(Unit::from_str("milliusdc").unwrap(), Unit::MilliUsdc);
        assert!(Unit::from_str("unknown").is_err());
    }

    #[test]
    fn test_unit_asset_mapping() {
        assert_eq!(Unit::Gwei.asset(), Asset::Eth);
        assert_eq!(Unit::MilliUsdc.asset(), Asset::Usdc);
    }

    #[test]
    fn test_amount_conversion() {
        let amount = Amount::from(1000);
        
        // Test Gwei conversion
        let gwei_u256 = Unit::Gwei.convert_amount_into_u256(amount);
        assert_eq!(gwei_u256, U256::from(1000) * U256::from(10).pow(9.into()));
        
        // Test MilliUsdc conversion
        let milliusdc_u256 = Unit::MilliUsdc.convert_amount_into_u256(amount);
        assert_eq!(milliusdc_u256, U256::from(1000) * U256::from(1000));
    }
}
