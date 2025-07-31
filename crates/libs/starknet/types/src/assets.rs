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
    WBtc,
    Usdc,
    Usdt,
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
            Asset::WBtc => "wbtc",
            Asset::Usdc => "usdc",
            Asset::Usdt => "usdt",
        }
    }

    pub fn precision(&self) -> u8 {
        match self {
            Asset::Strk | Asset::Eth => 18,
            Asset::WBtc => 8,
            Asset::Usdc | Asset::Usdt => 6,
        }
    }

    pub fn scale_factor(&self) -> U256 {
        match self {
            Asset::Strk | Asset::Eth => U256::from(1_000_000_000_000_000_000u64),
            Asset::WBtc => U256::from(100_000_000u64),
            Asset::Usdc | Asset::Usdt => U256::from(1_000_000u64),
        }
    }

    pub fn find_best_unit(&self) -> Unit {
        match self {
            Asset::Strk => Unit::MilliStrk,
            Asset::Eth => Unit::Gwei,
            Asset::WBtc => Unit::Satoshi,
            Asset::Usdc | Asset::Usdt => Unit::CentUsd,
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
        let scale_factor = match (self, unit) {
            (Asset::Usdc, Unit::CentUsd) => 10_000u64, // 1 USDC (1e6) = 100 cents, so divide by 1e4
            (Asset::Usdt, Unit::CentUsd) => 10_000u64, // Same for USDT
            _ => unit.scale_factor(),
        };

        let (quotient, rem) = asset_amount.div_mod(U256::from(scale_factor));

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
            "strk" => Ok(Asset::Strk),
            "eth" => Ok(Asset::Eth),
            "btc" => Ok(Asset::WBtc),
            "usdc" => Ok(Asset::Usdc),
            "usdt" => Ok(Asset::Usdt),
            _ => Err(AssetFromStrError),
        }
    }
}
