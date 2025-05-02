use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Unit;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    Strk,
    Eth,
}

impl core::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetToUnitConversionError {
    #[error("couldn't convert asset amount to unit: {0}")]
    AmountTooBigForU64(&'static str),
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Asset::Strk => "strk",
            Asset::Eth => "eth",
        }
    }

    pub fn precision(&self) -> U256 {
        match self {
            Asset::Strk | Asset::Eth => U256::from(1_000_000_000_000_000_000u64),
        }
    }

    fn find_best_unit_for_asset_amount(&self, _asset_amount: U256) -> Unit {
        match self {
            Asset::Strk => Unit::MilliStrk,
            Asset::Eth => Unit::Gwei,
        }
    }

    /// Convert an onchain amount of asset to a protocol amount of unit
    ///
    /// # WARNING
    /// The input amount HAS to be specified including on-chain precision.
    /// eg. 1 stark, should be passed as 1*10^18
    pub fn convert_to_amount_of_unit(
        &self,
        asset_amount: U256,
        unit: Unit,
    ) -> Result<(Amount, U256), AssetToUnitConversionError> {
        let (quotien, rem) = asset_amount.div_mod(U256::from(unit.conversion_rate()));

        Ok((
            Amount::from(
                u64::try_from(quotien).map_err(AssetToUnitConversionError::AmountTooBigForU64)?,
            ),
            rem,
        ))
    }

    /// Convert an onchain amount of asset to a protocol amount of the most appropriate unit
    ///
    /// # WARNING
    /// The input amount HAS to be specified including on-chain precision.
    /// eg. 1 stark, should be passed as 1*10^18
    pub fn convert_to_amount_and_unit(
        &self,
        asset_amount: U256,
    ) -> Result<(Amount, Unit, U256), AssetToUnitConversionError> {
        let unit = self.find_best_unit_for_asset_amount(asset_amount);
        let (amount, rem) = self.convert_to_amount_of_unit(asset_amount, unit)?;

        Ok((amount, unit, rem))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid asset")]
pub struct AssetFromStrError;

impl FromStr for Asset {
    type Err = AssetFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strk" => Ok(Asset::Strk),
            "eth" => Ok(Asset::Eth),
            _ => Err(AssetFromStrError),
        }
    }
}
