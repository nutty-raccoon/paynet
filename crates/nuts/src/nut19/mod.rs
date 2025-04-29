//! NUT-19: Cached Responses
//!
//! <https://github.com/cashubtc/nuts/blob/main/19.md>
//! We implement it slightly different due to our use of gRPC

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Mint settings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Settings {
    /// Number of seconds the responses are cached for
    pub ttl: Option<u64>,
}

/// Route path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Route {
    /// Mint
    Mint,
    /// Melt
    Melt,
    /// Swap
    Swap,
    /// Restore
    Restore,
}

pub const MINT: &str = "mint";
pub const SWAP: &str = "swap";
pub const MELT: &str = "melt";
pub const RESTORE: &str = "restore";

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Route::Mint => MINT,
                Route::Melt => MELT,
                Route::Swap => SWAP,
                Route::Restore => RESTORE,
            }
        )
    }
}

pub type CacheResponseKey = (Route, u64);

#[derive(Debug, thiserror::Error)]
pub enum PathFromStrError {
    #[error("Invalid version {0}")]
    InvalidVersion(String),
    #[error("Invalid")]
    InvalidUri,
    #[error("Invalid method {0}")]
    InvalidMethod(String),
    #[error("Invalid route {0}")]
    InvalidRoute(String),
}

impl FromStr for Route {
    type Err = PathFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            MINT => Ok(Self::Mint),
            SWAP => Ok(Self::Swap),
            MELT => Ok(Self::Melt),
            _ => Err(PathFromStrError::InvalidRoute(s.to_string())),
        }
    }
}

impl Serialize for Route {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Route {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::from_str(<&str>::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
