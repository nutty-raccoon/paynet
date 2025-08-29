// Largely copied from https://github.com/cashubtc/cdk
// Distributed under MIT license:
// Copyright (c) 2023-2024, thesimplekid (BSD-3-Clause)
// Copyright (c) 2024 Cashu Dev Kit Developers

mod amount;
mod types;

pub mod dhke;
pub mod nut00;
pub mod nut01;
pub mod nut02;
pub mod nut03;
pub mod nut04;
pub mod nut05;
pub mod nut06;
pub mod nut07;
#[cfg(feature = "nut13")]
pub mod nut13;
#[cfg(feature = "nut19")]
pub mod nut19;

pub use amount::*;
pub use types::*;

use bitcoin::secp256k1::{All, Secp256k1, rand};
use once_cell::sync::Lazy;

pub mod traits {
    use std::{
        fmt::{Debug, Display},
        hash::Hash,
        str::FromStr,
    };

    pub trait Method: Debug + Display + FromStr {}

    pub trait Asset: Copy + Clone + AsRef<str> + Hash {
        fn precision(&self) -> u8;
    }

    /// A cashu representation of an onchain asset
    ///
    /// An asset can have multiple possible unit representation (eg. one eth can be represented as wei or gwei).
    pub trait Unit:
        FromStr + AsRef<str> + Sized + Debug + Copy + Clone + Display + Into<u32> + Eq + PartialEq
    {
        type Asset: Asset;

        /// Verifies that an asset is compatible with this unit
        ///
        /// This check helps to catch accidental mismatches between units and assets early.
        fn is_asset_supported(&self, asset: Self::Asset) -> bool;
        /// Multiply the unit amount by 10^n to get the value in the corresponding asset precision
        fn asset_extra_precision(&self) -> u8;
        /// Returns the asset represented by this unit
        fn matching_asset(&self) -> Self::Asset;
    }

    #[cfg(test)]
    pub mod test_types {
        use std::{fmt::Display, str::FromStr};

        use crate::Error;

        use super::{Asset, Method, Unit};

        #[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum TestUnit {
            Sat,
            Msat,
            Usd,
            Eur,
        }

        #[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
        pub enum TestAsset {
            Btc,
            Usd,
            Eur,
        }

        impl AsRef<str> for TestAsset {
            fn as_ref(&self) -> &str {
                match self {
                    TestAsset::Btc => "BTC",
                    TestAsset::Usd => "eur",
                    TestAsset::Eur => "usd",
                }
            }
        }

        impl Asset for TestAsset {
            fn precision(&self) -> u8 {
                8
            }
        }

        impl Display for TestUnit {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self.as_ref(), f)
            }
        }

        impl From<TestUnit> for u32 {
            fn from(value: TestUnit) -> Self {
                match value {
                    TestUnit::Sat => 0,
                    TestUnit::Msat => 1,
                    TestUnit::Usd => 2,
                    TestUnit::Eur => 3,
                }
            }
        }

        impl FromStr for TestUnit {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let unit = match s {
                    "sat" => TestUnit::Sat,
                    "msat" => TestUnit::Msat,
                    "usd" => TestUnit::Usd,
                    "eur" => TestUnit::Eur,
                    _ => return Err(Error::CannotConvertUnits),
                };
                Ok(unit)
            }
        }

        impl Unit for TestUnit {
            type Asset = TestAsset;

            fn is_asset_supported(&self, asset: Self::Asset) -> bool {
                match asset {
                    TestAsset::Btc => true,
                    TestAsset::Usd => true,
                    TestAsset::Eur => true,
                }
            }

            fn asset_extra_precision(&self) -> u8 {
                1
            }

            fn matching_asset(&self) -> Self::Asset {
                match self {
                    TestUnit::Sat => TestAsset::Btc,
                    TestUnit::Msat => TestAsset::Btc,
                    TestUnit::Usd => TestAsset::Usd,
                    TestUnit::Eur => TestAsset::Eur,
                }
            }
        }

        impl AsRef<str> for TestUnit {
            fn as_ref(&self) -> &str {
                match self {
                    TestUnit::Sat => "sat",
                    TestUnit::Msat => "msat",
                    TestUnit::Usd => "usd",
                    TestUnit::Eur => "eur",
                }
            }
        }

        #[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum TestMethod {
            Bolt11,
        }

        impl Display for TestMethod {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        TestMethod::Bolt11 => "bolt11",
                    }
                )
            }
        }

        impl FromStr for TestMethod {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s == "bolt11" {
                    Ok(Self::Bolt11)
                } else {
                    Err("bad method value")
                }
            }
        }

        impl Method for TestMethod {}
    }
}

/// Secp256k1 global context
pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(|| {
    let mut ctx = Secp256k1::new();
    let mut rng = rand::thread_rng();
    ctx.randomize(&mut rng);
    ctx
});
