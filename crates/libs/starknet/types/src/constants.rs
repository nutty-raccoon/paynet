//! Network-specific Configuration Constants
//!
//! This module provides a centralized location for all network-specific constants
//! used throughout the application. By organizing constants into a single map indexed
//! by network identifier, we ensure consistent configuration across the application
//! and simplify network switching.
//!
//! The `phf` crate is used to create compile-time static maps, which guarantees
//! zero runtime overhead when accessing these constants.

use starknet_types_core::felt::Felt;

use crate::Asset;

#[derive(Debug, Clone)]
pub struct AssetsAddress([(Asset, Felt); 5]);

impl AssetsAddress {
    pub fn get_contract_address_for_asset(&self, asset: Asset) -> Option<Felt> {
        self.0
            .iter()
            .find(|(a, _)| asset == *a)
            .map(|(_, address)| *address)
    }

    pub fn get_asset_for_contract_address(&self, contract_address: Felt) -> Option<Asset> {
        self.0
            .iter()
            .find(|(_, a)| contract_address == *a)
            .map(|(asset, _)| *asset)
    }
}

/// Assets available on Starknet Sepolia testnet with their contract addresses
///
/// These addresses are network-specific and have been verified to be the official
/// token contracts.
const SEPOLIA_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress([
    (
        Asset::Strk,
        Felt::from_hex_unchecked(
            "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
        ),
    ),
    (
        Asset::Eth,
        Felt::from_hex_unchecked(
            "0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7",
        ),
    ),
    (
        Asset::WBtc,
        Felt::from_hex_unchecked(
            "0x03fe2b97c1fd336e750087d68b9b867997fd64a2661ff3ca5a7c771641e8e960",
        ),
    ),
    (
        Asset::Usdc,
        Felt::from_hex_unchecked(
            "0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8",
        ),
    ),
    (
        Asset::Usdt,
        Felt::from_hex_unchecked(
            "0x068f5c6a61780768455de69077e07e89787839bf8166decfbf92b645209c0fb8",
        ),
    ),
]);

/// Devnet assets with placeholder addresses 
const DEVNET_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress([
    (
        Asset::Strk,
        Felt::from_hex_unchecked(
            "0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
        ),
    ),
    (
        Asset::Eth,
        Felt::from_hex_unchecked(
            "0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7",
        ),
    ),
    (
        Asset::WBtc,
        Felt::from_hex_unchecked(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ),
    ),
    (
        Asset::Usdc,
        Felt::from_hex_unchecked(
            "0x0000000000000000000000000000000000000000000000000000000000000002",
        ),
    ),
    (
        Asset::Usdt,
        Felt::from_hex_unchecked(
            "0x0000000000000000000000000000000000000000000000000000000000000003",
        ),
    ),
]);

/// Top-level constants container for each network configuration
///
/// This structure groups related constants logically, making it easier to
/// add new networks or extend the configuration in the future.
#[derive(Debug, Clone)]
pub struct OnChainConstants {
    pub substreams: SubstreamsConstants,
    pub invoice_payment_contract_address: Felt,
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
    "SN_SEPOLIA" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 812115 },
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x019dce9fd974e01665968f94784db3e94daac279cdef4289133d60954e90298a"),
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
    "SN_DEVNET" => OnChainConstants {
        substreams: SubstreamsConstants { starting_block: 0 },
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x026b2c472aa4ea32fc12f6c44707712552eff4aac48dd75c870e79b8a3fb676e"),
        assets_contract_address: DEVNET_ASSETS_ADDRESSES,
    },
};
