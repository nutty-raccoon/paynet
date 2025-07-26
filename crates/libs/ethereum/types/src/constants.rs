use phf::phf_map;
use primitive_types::H160;
use crate::{Asset, ChainId, EthereumAddress};

/// Contract addresses for different assets on each network
#[derive(Debug, Clone)]
pub struct AssetsAddress {
    pub eth: Option<EthereumAddress>,
    pub usdc: Option<EthereumAddress>,
    pub usdt: Option<EthereumAddress>,
}

impl AssetsAddress {
    /// Get the contract address for a specific asset
    pub fn get_contract_address_for_asset(&self, asset: Asset) -> Option<EthereumAddress> {
        match asset {
            Asset::Eth => self.eth,
            Asset::Usdc => self.usdc,
            Asset::Usdt => self.usdt,
        }
    }
}

/// Substreams-specific configuration for data streaming
#[derive(Debug, Clone)]
pub struct SubstreamsConstants {
    pub starting_block: u64,
}

/// Top-level constants container for each network configuration
///
/// This structure groups related constants logically, making it easier to
/// add new networks or extend the configuration in the future.
#[derive(Debug, Clone)]
pub struct OnChainConstants {
    pub substreams: SubstreamsConstants,
    pub invoice_payment_contract_address: EthereumAddress,
    pub assets_contract_address: AssetsAddress,
}

/// Asset addresses for Ethereum Sepolia testnet
const SEPOLIA_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress {
    // ETH is native, no contract address needed
    eth: None,
    // USDC on Sepolia testnet
    usdc: Some(H160([
        0x1c, 0x7D, 0x4B, 0x19, 0x6C, 0xb0, 0xC7, 0xB0, 0x1d, 0x3A,
        0xc0, 0x49, 0xdf, 0x3b, 0x4f, 0x0c, 0x3f, 0x2f, 0x6d, 0x9c
    ])),
    // USDT on Sepolia testnet (example address)
    usdt: Some(H160([
        0x7b, 0x79, 0x99, 0x5e, 0x5f, 0x79, 0x3A, 0x07, 0xBc, 0x00,
        0xc2, 0x1A, 0xD4, 0x9f, 0x25, 0x53, 0x9f, 0x3e, 0x5c, 0x4f
    ])),
};

/// Asset addresses for Ethereum Mainnet
const MAINNET_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress {
    // ETH is native, no contract address needed
    eth: None,
    // USDC on Mainnet
    usdc: Some(H160([
        0xA0, 0xb8, 0x69, 0x91, 0xc6, 0xcc, 0x5e, 0x2b, 0x0c, 0x0b,
        0x76, 0x2d, 0x89, 0x3a, 0x0b, 0x63, 0x0b, 0x69, 0x29, 0x18
    ])),
    // USDT on Mainnet
    usdt: Some(H160([
        0xdA, 0xC1, 0x7F, 0x95, 0x8D, 0x2e, 0xe5, 0x23, 0xa2, 0x20,
        0x62, 0x06, 0x99, 0x45, 0x97, 0xC1, 0x3D, 0x83, 0x1e, 0xc7
    ])),
};

/// Asset addresses for local devnet
const DEVNET_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress {
    // ETH is native, no contract address needed
    eth: None,
    // Mock USDC contract for devnet
    usdc: Some(H160([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01
    ])),
    // Mock USDT contract for devnet
    usdt: Some(H160([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02
    ])),
};

/// Map of all supported networks and their corresponding constants
///
/// This is the primary entry point for accessing network-specific configuration.
/// New networks can be added here without modifying the rest of the codebase.
pub static ON_CHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf_map! {
    "ETH_SEPOLIA" => OnChainConstants {
        // Starting block for Sepolia testnet (example block)
        substreams: SubstreamsConstants { starting_block: 4000000 },
        // Invoice payment contract address on Sepolia (placeholder)
        invoice_payment_contract_address: H160([
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34,
            0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78
        ]),
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
    "ETH_MAINNET" => OnChainConstants {
        // Starting block for Mainnet (example block)
        substreams: SubstreamsConstants { starting_block: 18000000 },
        // Invoice payment contract address on Mainnet (placeholder)
        invoice_payment_contract_address: H160([
            0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
            0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01
        ]),
        assets_contract_address: MAINNET_ASSETS_ADDRESSES,
    },
    "ETH_DEVNET" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        // Invoice payment contract address for devnet (placeholder)
        invoice_payment_contract_address: H160([
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11
        ]),
        assets_contract_address: DEVNET_ASSETS_ADDRESSES,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_chain_constants_access() {
        let sepolia_constants = ON_CHAIN_CONSTANTS.get("ETH_SEPOLIA").unwrap();
        assert_eq!(sepolia_constants.substreams.starting_block, 4000000);
        
        let mainnet_constants = ON_CHAIN_CONSTANTS.get("ETH_MAINNET").unwrap();
        assert_eq!(mainnet_constants.substreams.starting_block, 18000000);
        
        let devnet_constants = ON_CHAIN_CONSTANTS.get("ETH_DEVNET").unwrap();
        assert_eq!(devnet_constants.substreams.starting_block, 0);
    }

    #[test]
    fn test_assets_address_lookup() {
        let sepolia_constants = ON_CHAIN_CONSTANTS.get("ETH_SEPOLIA").unwrap();
        
        // ETH should be None (native asset)
        assert!(sepolia_constants.assets_contract_address.get_contract_address_for_asset(Asset::Eth).is_none());
        
        // USDC should have an address
        assert!(sepolia_constants.assets_contract_address.get_contract_address_for_asset(Asset::Usdc).is_some());
        
        // USDT should have an address
        assert!(sepolia_constants.assets_contract_address.get_contract_address_for_asset(Asset::Usdt).is_some());
    }
}
