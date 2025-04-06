use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "&'static str", try_from = "&str")]
pub struct Method;

const STARKNET_METHOD: &str = "starknet";

impl From<Method> for &'static str {
    fn from(_value: Method) -> Self {
        STARKNET_METHOD
    }
}

#[derive(Debug, Error)]
#[error("Invalid value for type `Method`")]
pub struct MethodFromStrError;

impl FromStr for Method {
    type Err = MethodFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == STARKNET_METHOD {
            Ok(Self)
        } else {
            Err(MethodFromStrError)
        }
    }
}

impl TryFrom<&str> for Method {
    type Error = MethodFromStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Display::fmt(STARKNET_METHOD, f)
    }
}
