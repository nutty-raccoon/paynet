//! NUT-19: Cached Responses
//!
//! <https://github.com/cashubtc/nuts/blob/main/19.md>

use std::{fmt::Display, str::FromStr};

use crate::traits;

use serde::{Deserialize, Serialize};

/// Mint settings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Settings {
    /// Number of seconds the responses are cached for
    pub ttl: Option<u64>,
}

/// List of the methods and paths for which caching is enabled
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CachedEndpoint<M: traits::Method> {
    /// HTTP Method
    pub method: HttpMethod,
    /// Route path
    pub path: Path<M>,
}

impl<M: traits::Method> CachedEndpoint<M> {
    /// Create [`CachedEndpoint`]
    pub fn new(method: HttpMethod, path: Path<M>) -> Self {
        Self { method, path }
    }
}

/// HTTP method
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// Get
    Get,
    /// POST
    Post,
}

/// Route path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Path<M> {
    /// Mint
    Mint(M),
    /// Melt
    Melt(M),
    /// Swap
    Swap,
}

pub const V1_MINT: &str = "/v1/mint/";
pub const V1_SWAP: &str = "/v1/swap";
pub const V1_MELT: &str = "/v1/melt/";

impl<M: Display> Display for Path<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Path::Mint(m) => format!("{}{}", V1_MINT, m),
                Path::Melt(m) => format!("{}{}", V1_MELT, m),
                Path::Swap => V1_SWAP.to_string(),
            }
        )
    }
}

pub type CacheResponseKey<M> = (Path<M>, String);

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

impl<M: FromStr> FromStr for Path<M> {
    type Err = PathFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == V1_SWAP {
            return Ok(Self::Swap);
        }

        let mut splits = s.split('/');
        // Because uri is absolute it starts with a '/' making the first element of splis empty
        let _ = splits.next().ok_or(PathFromStrError::InvalidUri)?;
        let version = splits.next().ok_or(PathFromStrError::InvalidUri)?;
        if version != "v1" {
            return Err(PathFromStrError::InvalidVersion(version.to_string()));
        }
        let route = splits.next().ok_or(PathFromStrError::InvalidUri)?;
        let method = splits.next().ok_or(PathFromStrError::InvalidUri)?;
        let method =
            M::from_str(method).map_err(|_| PathFromStrError::InvalidMethod(method.to_string()))?;
        match route {
            "mint" => Ok(Self::Mint(method)),
            "melt" => Ok(Self::Melt(method)),
            _ => Err(PathFromStrError::InvalidRoute(route.to_string())),
        }
    }
}

impl<M: Display> Serialize for Path<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de, M: FromStr> Deserialize<'de> for Path<M> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::from_str(<&str>::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
