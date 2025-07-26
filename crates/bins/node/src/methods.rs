use std::str::FromStr;

use serde::{Deserialize, Serialize};
use starknet_types::STARKNET_STR;

const ETHEREUM_STR: &str = "ethereum";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Starknet,
    Ethereum,
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Method::Starknet => Serialize::serialize(STARKNET_STR, serializer),
            Method::Ethereum => Serialize::serialize(ETHEREUM_STR, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        match s {
            STARKNET_STR => Ok(Method::Starknet),
            ETHEREUM_STR => Ok(Method::Ethereum),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &"a supported method",
            )),
        }
    }
}

impl core::fmt::Display for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Method::Starknet => core::fmt::Display::fmt(STARKNET_STR, f),
            Method::Ethereum => core::fmt::Display::fmt(ETHEREUM_STR, f),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("bad method")]
pub struct FromStrError;

impl FromStr for Method {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            STARKNET_STR => Ok(Self::Starknet),
            ETHEREUM_STR => Ok(Self::Ethereum),
            _ => Err(FromStrError),
        }
    }
}

impl nuts::traits::Method for Method {}
