use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    Starknet,
}

impl Serialize for Method {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Method::Starknet => Serialize::serialize(&starknet_types::Method, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Method {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <starknet_types::Method as Deserialize>::deserialize(deserializer).map(|_| Method::Starknet)
    }
}

impl core::fmt::Display for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
        if <starknet_types::Method as FromStr>::from_str(s).is_ok() {
            Ok(Self::Starknet)
        } else {
            Err(FromStrError)
        }
    }
}

impl nuts::traits::Method for Method {}
