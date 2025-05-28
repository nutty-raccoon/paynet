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
    pub apibara: ApibaraConstants,
    pub invoice_payment_contract_address: Felt,
    pub assets_contract_address: AssetsAddress,
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
    "SN_SEPOLIA" =>  OnChainConstants {
        // Starting block is the one which contains the invoice_payment_contract deployment
        // Tx: 0x0582cb60c2fc97fd9fbb18a818197611e1971498a3e5a34272d7072d70a009f3
        apibara: ApibaraConstants { data_stream_uri:  Some("http://sepolia.starknet.a5a.ch"), starting_block: 808660 },
        //
        // Declaration
        //
        // Declaring Cairo 1 class: 0x06542863bc6124f0b00d3e87837bef7a7c65b8701f33be3c4e9a5235e99aad85
        // Contract declaration transaction: 0x0789769d3e488c721813cce9faf56bbf89566260c976f8bdf013433c3e9b2686
        // Class hash declared: 0x06542863bc6124f0b00d3e87837bef7a7c65b8701f33be3c4e9a5235e99aad85
        //
        // Deployment
        //
        // Deploying class 0x06542863bc6124f0b00d3e87837bef7a7c65b8701f33be3c4e9a5235e99aad85
        // The contract will be deployed at address 0x03ea260a0d19a073d2b5b97e7673605e164fbe2660d76953254060dda0c38124
        // Contract deployment transaction: 0x0582cb60c2fc97fd9fbb18a818197611e1971498a3e5a34272d7072d70a009f3
        // Contract deployed: 0x03ea260a0d19a073d2b5b97e7673605e164fbe2660d76953254060dda0c38124
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x03ea260a0d19a073d2b5b97e7673605e164fbe2660d76953254060dda0c38124"),
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
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
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x076d2f676fb9d9a4323307f723df93f47454729b8e6d716faf7defd620eb5000"),
        // The default starknet-devnet config reuses Sepolia asset addresses
        // TODO: will only work for `eth` and `strk` assets. So we will change it later on.
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
};
