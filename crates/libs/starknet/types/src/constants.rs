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
pub struct AssetsAddress([(Asset, Felt); 2]);

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
    "SN_SEPOLIA" =>  OnChainConstants {
        // Starting block is the one which contains the invoice_payment_contract deployment
        // Tx: 0x0582cb60c2fc97fd9fbb18a818197611e1971498a3e5a34272d7072d70a009f3
        substreams: SubstreamsConstants {  starting_block: 812115 },
        //
        // Declaration
        //
        // Declaring Cairo 1 class: 0x0476fd5052392e3f46a384d8d38674d0727714af1e44583effe1ed6c1700da37
        // Contract declaration transaction: 0x020e418cf124652a2995dc1072d9c0944aa57bac2e25156cd89bec85db4a546e
        // Class hash declared: 0x0476fd5052392e3f46a384d8d38674d0727714af1e44583effe1ed6c1700da37
        //
        // Deployment
        //
        // Deploying class 0x0476fd5052392e3f46a384d8d38674d0727714af1e44583effe1ed6c1700da37
        // The contract will be deployed at address 0x019dce9fd974e01665968f94784db3e94daac279cdef4289133d60954e90298a
        // Contract deployment transaction: 0x03a61d43d856d59a28d9efbd5d264825408781cfb63400ab437b19180f523ad5
        // Contract deployed: 0x019dce9fd974e01665968f94784db3e94daac279cdef4289133d60954e90298a
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x019dce9fd974e01665968f94784db3e94daac279cdef4289133d60954e90298a"),
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
    "SN_DEVNET" =>  OnChainConstants {
        substreams: SubstreamsConstants {
            starting_block: 0
        },
        // This address is guaranted to be correct, if and only if,
        // you are using our `starknet-on-chain-setup` rust deployment executable.
        // It is automaticaly used when setting up the network using this repo's `docker-compose.yml`
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x026b2c472aa4ea32fc12f6c44707712552eff4aac48dd75c870e79b8a3fb676e"),
        // The default starknet-devnet config reuses Sepolia asset addresses
        // TODO: will only work for `eth` and `strk` assets. So we will change it later on.
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
};
