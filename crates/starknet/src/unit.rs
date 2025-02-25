use std::str::FromStr;

use nuts::Amount;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{Asset, Error, StarknetU256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Strk,
}

impl Unit {
    pub fn asset(&self) -> Asset {
        match self {
            Unit::Strk => Asset::Strk,
        }
    }
}

// Used for derivation path
impl From<Unit> for u32 {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Strk => 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid value for enum `Unit`")]
pub struct UnitFromStrError;

impl FromStr for Unit {
    type Err = UnitFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unit = match s {
            "strk" => Self::Strk,
            _ => return Err(UnitFromStrError),
        };

        Ok(unit)
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(
            match self {
                Unit::Strk => "strk",
            },
            f,
        )
    }
}

impl nuts::traits::Unit for Unit {}

// e-3 strk
const STRK_UNIT_TO_ASSET_CONVERSION_RATE: u64 = 1_000_000_000_000_000;

impl Unit {
    pub fn convert_amount_into_u256(&self, amount: Amount) -> StarknetU256 {
        match self {
            Unit::Strk => StarknetU256::from(
                U256::from(u64::from(amount)) * U256::from(STRK_UNIT_TO_ASSET_CONVERSION_RATE),
            ),
        }
    }

    pub fn convert_u256_into_amount(
        &self,
        amount: StarknetU256,
    ) -> Result<(Amount, StarknetU256), Error> {
        match self {
            Unit::Strk => {
                let (quotient, rem) = primitive_types::U256::from(&amount)
                    .div_mod(U256::from(STRK_UNIT_TO_ASSET_CONVERSION_RATE));
                Ok((
                    Amount::from(
                        u64::try_from(quotient)
                            .map_err(|_| Error::StarknetAmountTooHigh(*self, amount))?,
                    ),
                    StarknetU256::from(rem),
                ))
            }
        }
    }

    pub fn is_asset_supported(&self, asset: Asset) -> bool {
        match (self, asset) {
            (Unit::Strk, Asset::Strk) => true,
        }
    }
}
