use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    #[cfg(any(feature = "mock", feature = "starknet"))]
    Starknet,
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            #[cfg(any(feature = "mock", feature = "starknet"))]
            Method::Starknet => Serialize::serialize(&starknet_types::Method, serializer),
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
            #[cfg(any(feature = "mock", feature = "starknet"))]
            "starknet" => Ok(Method::Starknet),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &"a supported method",
            )),
        }
    }
}

impl core::fmt::Display for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            #[cfg(any(feature = "mock", feature = "starknet"))]
            Method::Starknet => core::fmt::Display::fmt(&starknet_types::Method, f),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("bad value")]
pub struct FromStrError;

impl FromStr for Method {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[cfg(any(feature = "mock", feature = "starknet"))]
        if <starknet_types::Method as FromStr>::from_str(s).is_ok() {
            return Ok(Self::Starknet);
        };

        Err(FromStrError)
    }
}

impl nuts::traits::Method for Method {}
