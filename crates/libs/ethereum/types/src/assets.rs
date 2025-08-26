use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Unit;

/// Represents a supported blockchain asset.
///
/// In this Ethereum-only context, we only support ETH.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    /// Ethereum (ETH)
    Weth,
}

impl core::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error that occurs when converting a U256 blockchain value to a smaller Amount type.
#[derive(Debug, thiserror::Error)]
pub enum AssetToUnitConversionError {
    #[error("couldn't convert asset amount to unit: {0}")]
    AmountTooBigForU64(&'static str),
}

impl Asset {
    /// Returns the canonical string representation of the asset.
    pub fn as_str(&self) -> &str {
        match self {
            Asset::Weth => "weth",
        }
    }

    /// Returns the decimal precision of the asset.
    pub fn precision(&self) -> u8 {
        // ETH uses 18 decimal places
        18
    }

    /// Returns the scale factor to convert 1 ETH into wei.
    pub fn scale_factor(&self) -> U256 {
        U256::from(1_000_000_000_000_000_000u64) // 10^18 wei = 1 ETH
    }

    /// Picks the most user-friendly unit for representing this asset.
    pub fn find_best_unit(&self) -> Unit {
        Unit::Gwei
    }

    /// Converts an on-chain asset amount into a protocol amount of a given unit.
    ///
    /// Returns the Amount (as u64) and the remainder in native U256 form.
    ///
    /// # Warning
    /// `asset_amount` must be specified in on-chain precision (e.g. 1 ETH = 10^18).
    pub fn convert_to_amount_of_unit(
        &self,
        asset_amount: U256,
        unit: Unit,
    ) -> Result<(Amount, U256), AssetToUnitConversionError> {
        let (quotient, rem) = asset_amount.div_mod(U256::from(unit.scale_factor()));
        Ok((
            Amount::from(
                u64::try_from(quotient).map_err(AssetToUnitConversionError::AmountTooBigForU64)?,
            ),
            rem,
        ))
    }

    /// Converts an on-chain asset amount into the most appropriate unit.
    ///
    /// Returns (Amount, Unit, remainder in wei).
    pub fn convert_to_amount_and_unit(
        &self,
        asset_amount: U256,
    ) -> Result<(Amount, Unit, U256), AssetToUnitConversionError> {
        let unit = self.find_best_unit();
        let (amount, rem) = self.convert_to_amount_of_unit(asset_amount, unit)?;
        Ok((amount, unit, rem))
    }
}

/// Error returned when parsing an invalid string into an `Asset`.
#[derive(Debug, thiserror::Error)]
#[error("invalid asset")]
pub struct AssetFromStrError;

impl FromStr for Asset {
    type Err = AssetFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "weth" => Ok(Asset::Weth),
            _ => Err(AssetFromStrError),
        }
    }
}
