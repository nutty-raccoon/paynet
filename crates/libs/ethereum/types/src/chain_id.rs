use std::str::FromStr;
use serde::{Deserialize, Serialize};

// Constants representing predefined Ethereum networks
const ETH_MAINNET: &str = "ETH_MAINNET";
const ETH_SEPOLIA: &str = "ETH_SEPOLIA";
const ETH_DEVNET: &str = "ETH_DEVNET";

/// Represents supported Ethereum chain IDs
///
/// This enum provides a type-safe way to handle different Ethereum networks
/// and their corresponding chain IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    /// Ethereum Mainnet (Chain ID: 1)
    Mainnet,
    /// Ethereum Sepolia Testnet (Chain ID: 11155111)
    Sepolia,
    /// Local development network (Chain ID: 31337)
    Devnet,
}

impl ChainId {
    /// Returns the numeric chain ID for the network
    pub fn chain_id(&self) -> u64 {
        match self {
            ChainId::Mainnet => 1,
            ChainId::Sepolia => 11155111,
            ChainId::Devnet => 31337,
        }
    }

    /// Returns the network name as used in configuration
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainId::Mainnet => ETH_MAINNET,
            ChainId::Sepolia => ETH_SEPOLIA,
            ChainId::Devnet => ETH_DEVNET,
        }
    }

    /// Returns a human-readable name for the network
    pub fn display_name(&self) -> &'static str {
        match self {
            ChainId::Mainnet => "Ethereum Mainnet",
            ChainId::Sepolia => "Ethereum Sepolia",
            ChainId::Devnet => "Ethereum Devnet",
        }
    }

    /// Returns whether this is a testnet
    pub fn is_testnet(&self) -> bool {
        matches!(self, ChainId::Sepolia | ChainId::Devnet)
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChainIdFromStrError {
    #[error("Unknown chain ID: {0}")]
    UnknownChainId(String),
}

impl FromStr for ChainId {
    type Err = ChainIdFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ETH_MAINNET => Ok(ChainId::Mainnet),
            ETH_SEPOLIA => Ok(ChainId::Sepolia),
            ETH_DEVNET => Ok(ChainId::Devnet),
            _ => Err(ChainIdFromStrError::UnknownChainId(s.to_string())),
        }
    }
}

impl From<ChainId> for u64 {
    fn from(chain_id: ChainId) -> Self {
        chain_id.chain_id()
    }
}

impl TryFrom<u64> for ChainId {
    type Error = ChainIdFromStrError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ChainId::Mainnet),
            11155111 => Ok(ChainId::Sepolia),
            31337 => Ok(ChainId::Devnet),
            _ => Err(ChainIdFromStrError::UnknownChainId(value.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_from_str() {
        assert_eq!(ChainId::from_str("ETH_MAINNET").unwrap(), ChainId::Mainnet);
        assert_eq!(ChainId::from_str("ETH_SEPOLIA").unwrap(), ChainId::Sepolia);
        assert_eq!(ChainId::from_str("ETH_DEVNET").unwrap(), ChainId::Devnet);
        assert!(ChainId::from_str("UNKNOWN").is_err());
    }

    #[test]
    fn test_chain_id_numeric() {
        assert_eq!(ChainId::Mainnet.chain_id(), 1);
        assert_eq!(ChainId::Sepolia.chain_id(), 11155111);
        assert_eq!(ChainId::Devnet.chain_id(), 31337);
    }

    #[test]
    fn test_chain_id_conversion() {
        assert_eq!(u64::from(ChainId::Mainnet), 1);
        assert_eq!(ChainId::try_from(1).unwrap(), ChainId::Mainnet);
        assert_eq!(ChainId::try_from(11155111).unwrap(), ChainId::Sepolia);
        assert!(ChainId::try_from(999).is_err());
    }

    #[test]
    fn test_is_testnet() {
        assert!(!ChainId::Mainnet.is_testnet());
        assert!(ChainId::Sepolia.is_testnet());
        assert!(ChainId::Devnet.is_testnet());
    }
}
