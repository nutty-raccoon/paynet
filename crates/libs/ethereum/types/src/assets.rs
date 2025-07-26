use std::str::FromStr;
use serde::{Deserialize, Serialize};
use primitive_types::U256;

/// Supported Ethereum assets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Asset {
    Eth,
    Usdc,
    Usdt,
}

impl Asset {
    /// Returns the scale factor for the asset (10^decimals)
    pub fn scale_factor(&self) -> U256 {
        match self {
            Asset::Eth => U256::from(10).pow(18.into()), // 18 decimals
            Asset::Usdc => U256::from(10).pow(6.into()),  // 6 decimals
            Asset::Usdt => U256::from(10).pow(6.into()),  // 6 decimals
        }
    }

    /// Returns the number of decimals for the asset
    pub fn decimals(&self) -> u8 {
        match self {
            Asset::Eth => 18,
            Asset::Usdc => 6,
            Asset::Usdt => 6,
        }
    }

    /// Returns the asset symbol as a string
    pub fn symbol(&self) -> &'static str {
        match self {
            Asset::Eth => "ETH",
            Asset::Usdc => "USDC",
            Asset::Usdt => "USDT",
        }
    }
}

impl AsRef<str> for Asset {
    fn as_ref(&self) -> &str {
        match self {
            Asset::Eth => "eth",
            Asset::Usdc => "usdc",
            Asset::Usdt => "usdt",
        }
    }
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetFromStrError {
    #[error("Unknown asset: {0}")]
    UnknownAsset(String),
}

impl FromStr for Asset {
    type Err = AssetFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eth" => Ok(Asset::Eth),
            "usdc" => Ok(Asset::Usdc),
            "usdt" => Ok(Asset::Usdt),
            _ => Err(AssetFromStrError::UnknownAsset(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_from_str() {
        assert_eq!(Asset::from_str("eth").unwrap(), Asset::Eth);
        assert_eq!(Asset::from_str("ETH").unwrap(), Asset::Eth);
        assert_eq!(Asset::from_str("usdc").unwrap(), Asset::Usdc);
        assert_eq!(Asset::from_str("USDT").unwrap(), Asset::Usdt);
        assert!(Asset::from_str("unknown").is_err());
    }

    #[test]
    fn test_asset_scale_factor() {
        assert_eq!(Asset::Eth.scale_factor(), U256::from(10).pow(18.into()));
        assert_eq!(Asset::Usdc.scale_factor(), U256::from(10).pow(6.into()));
        assert_eq!(Asset::Usdt.scale_factor(), U256::from(10).pow(6.into()));
    }

    #[test]
    fn test_asset_decimals() {
        assert_eq!(Asset::Eth.decimals(), 18);
        assert_eq!(Asset::Usdc.decimals(), 6);
        assert_eq!(Asset::Usdt.decimals(), 6);
    }
}
