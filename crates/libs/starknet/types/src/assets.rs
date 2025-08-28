use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::Unit;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    Strk,
    Eth,
    WBtc,
    UsdC,
    UsdT,
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

const STRK_STR: &str = "strk";
const ETH_STR: &str = "eth";
const WBTC_STR: &str = "wbtc";
const USDT_STR: &str = "usdt";
const USDC_STR: &str = "usdc";

impl Asset {
    pub const fn as_str(&self) -> &str {
        match self {
            Asset::Strk => STRK_STR,
            Asset::Eth => ETH_STR,
            Asset::WBtc => WBTC_STR,
            Asset::UsdC => USDC_STR,
            Asset::UsdT => USDT_STR,
        }
    }

    pub fn scale_factor(&self) -> U256 {
        match self {
            Asset::Strk | Asset::Eth => U256::from(1_000_000_000_000_000_000u64),
            Asset::WBtc => U256::from(100_000_000u64),
            Asset::UsdC | Asset::UsdT => U256::from(1_000_000u64),
        }
    }

    pub fn find_best_unit(&self) -> Unit {
        match self {
            Asset::Strk => Unit::MilliStrk,
            Asset::Eth => Unit::Gwei,
            Asset::WBtc => Unit::Satoshi,
            Asset::UsdC => Unit::MicroUsdC,
            Asset::UsdT => Unit::MicroUsdT,
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
        let (quotient, rem) = asset_amount.div_mod(U256::from(unit.scale_factor()));

        Ok((
            Amount::from(
                u64::try_from(quotient).map_err(AssetToUnitConversionError::AmountTooBigForU64)?,
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
        let unit = self.find_best_unit();
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
        match s.to_lowercase().as_str() {
            STRK_STR => Ok(Asset::Strk),
            ETH_STR => Ok(Asset::Eth),
            WBTC_STR => Ok(Asset::WBtc),
            USDC_STR => Ok(Asset::UsdC),
            USDT_STR => Ok(Asset::UsdT),
            _ => Err(AssetFromStrError),
        }
    }
}

impl AsRef<str> for Asset {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl nuts::traits::Asset for Asset {
    fn precision(&self) -> u8 {
        match self {
            Asset::Strk | Asset::Eth => 18,
            Asset::WBtc => 8,
            Asset::UsdC | Asset::UsdT => 6,
        }
    }
}
