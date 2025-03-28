//! Network-specific Configuration Constants
//!
//! This module provides a centralized location for all network-specific constants
//! used throughout the application. By organizing constants into a single map indexed
//! by network identifier, we ensure consistent configuration across the application
//! and simplify network switching.
//!
//! The `phf` crate is used to create compile-time static maps, which guarantees
//! zero runtime overhead when accessing these constants.

use phf::phf_map;
use starknet_types_core::felt::Felt;

type AssetsMap = phf::Map<&'static str, Felt>;

/// Assets available on Starknet Sepolia testnet with their contract addresses
///
/// These addresses are network-specific and have been verified to be the official
/// token contracts.
static SEPOLIA_ASSETS_ADDRESSES: AssetsMap = phf_map! {
    "strk" => Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"),
    "eth" => Felt::from_hex_unchecked("0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7"),
};

/// Top-level constants container for each network configuration
///
/// This structure groups related constants logically, making it easier to
/// add new networks or extend the configuration in the future.
#[derive(Debug, Clone)]
pub struct OnChainConstants {
    pub apibara: ApibaraConstants,
    pub invoice_payment_contract_address: Felt,
    pub assets_contract_address: &'static phf::Map<&'static str, Felt>,
}

/// Apibara-specific configuration for data streaming
///
/// Apibara is used to index and stream blockchain events. Some networks
/// may not have Apibara support, hence the `Option` type for the URI.
#[derive(Debug, Clone)]
pub struct ApibaraConstants {
    pub data_stream_uri: Option<&'static str>,
    pub starting_block: u64,
}

/// Map of all supported networks and their corresponding constants
///
/// This is the primary entry point for accessing network-specific configuration.
/// New networks can be added here without modifying the rest of the codebase.
pub static ON_CHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf::phf_map! {
    // TODO: add sepolia
    // "SN_SEPOLIA" =>  OnChainConstants {
    //     apibara: ApibaraConstants { data_stream_uri:  Some("http://sepolia.starknet.a5a.ch"), starting_block: 0 },
    //     invoice_payment_contract_address: todo!(), // Not deployed atm
    //     assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    // },
    "SN_DEVNET" =>  OnChainConstants {
        apibara: ApibaraConstants {
            // No fixed Apibara indexer for devnet
            // we will read it's value at runtime
            // from `DNA_URI` env variable
            data_stream_uri: None,
            starting_block: 0
        },
        // This address is guaranted to be correct, if and only if,
        // you are using our `starknet-on-chain-setup` rust deployment executable.
        // It is automaticaly used when setting up the network using this repo's `docker-compose.yml`
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x074e3cbebe007eb4732706bec58067da01d16c0d252d763843c76612c69a4e9a"),
        // The default starknet-devnet config reuses Sepolia asset addresses
        // TODO: will only work for `eth` and `strk` assets. So we will change it later on.
        assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    },
};
