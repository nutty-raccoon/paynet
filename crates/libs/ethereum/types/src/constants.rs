//! Network-specific Configuration Constants
//!
//! This module provides a centralized location for all network-specific constants
//! used throughout the application. By organizing constants into a single map indexed
//! by network identifier, we ensure consistent configuration across the application
//! and simplify network switching.
//!
//! The `phf` crate is used to create compile-time static maps, which guarantees
//! zero runtime overhead when accessing these constants.

use ethers::types::Address;
use std::str::FromStr;

use crate::Asset;

#[derive(Debug, Clone)]
pub struct AssetsAddress(pub [(Asset, &'static str); 1]);

impl AssetsAddress {
    pub fn get_contract_address_for_asset(&self, asset: Asset) -> Option<&'static str> {
        self.0
            .iter()
            .find(|(a, _)| *a == asset)
            .map(|(_, addr)| *addr)
    }

    pub fn get_asset_for_contract_address(&self, contract_address: Address) -> Option<Asset> {
        self.0.iter().find_map(|(asset, addr_str)| {
            Address::from_str(addr_str)
                .ok()
                .filter(|parsed| *parsed == contract_address)
                .map(|_| *asset)
        })
    }
}

const SEPOLIA_ASSETS_ADDRESSES: AssetsAddress =
    AssetsAddress([(Asset::Weth, "0xdd13E55209Fd76AfE204dBda4007C227904f0a81")]);

const HOLESKY_ASSETS_ADDRESSES: AssetsAddress =
    AssetsAddress([(Asset::Weth, "0x94373a4919B3240D86eA41593D5eBa789FEF3848")]);

const MAINNET_ASSETS_ADDRESSES: AssetsAddress =
    AssetsAddress([(Asset::Weth, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")]);

const DEVNET_ASSETS_ADDRESSES: AssetsAddress =
    AssetsAddress([(Asset::Weth, "0x0000000000000000000000000000000000000000")]);

/// Top-level constants container for each network configuration
///
/// This structure groups related constants logically, making it easier to
/// add new networks or extend the configuration in the future.
pub struct OnChainConstants {
    pub substreams: SubstreamsConstants,
    pub invoice_payment_contract_address: &'static str,
    pub assets_contract_address: AssetsAddress,
}

/// Substreams-specific configuration for data streaming
#[derive(Debug, Clone)]
pub struct SubstreamsConstants {
    pub starting_block: u64,
}

/// Map of all supported networks and their corresponding constants
///
/// This is the primary entry point for accessing network-specific configuration.
/// New networks can be added here without modifying the rest of the codebase.
pub static ON_CHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf::phf_map! {
    "1" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        invoice_payment_contract_address: "0x0000000000000000000000000000000000000000",
        assets_contract_address: MAINNET_ASSETS_ADDRESSES,
    },
    "11155111" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        invoice_payment_contract_address: "0x503028d1f0c7a55d49c872745bb99dac084f959c",
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
    "17000" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        invoice_payment_contract_address: "0x0000000000000000000000000000000000000000",
        assets_contract_address: HOLESKY_ASSETS_ADDRESSES,
    },
    "1337" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        invoice_payment_contract_address: "0x0000000000000000000000000000000000000000",
        assets_contract_address: DEVNET_ASSETS_ADDRESSES,
    },
};
