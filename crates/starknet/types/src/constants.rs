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
        // Tx: 0x02ffa6a366b7224ed8fcdc4fda5afbd3f92266e478436d6b156e060862d5440f
        apibara: ApibaraConstants { data_stream_uri:  Some("http://sepolia.starknet.a5a.ch"), starting_block: 735060 },
        // Done with starkli 0.4.1 (b4223ee)
        //
        // Declaration
        //
        // Declaring Cairo 1 class: 0x0044b7358648f0c0bef25b4e36609642733275fef1b9253e21033d3d72403250
        // Compiling Sierra class to CASM with compiler version 2.11.4...
        // CASM class hash: 0x04a557923b9e5be32642cbbdca690af4896aede90a4765d94efc583113a48535
        // Contract declaration transaction: 0x031ada53541e88a431efc8a7eed44e2a62960e9ded11a8710ffb4b18a348ce75
        // Class hash declared: 0x0044b7358648f0c0bef25b4e36609642733275fef1b9253e21033d3d72403250
        //
        // Deployment
        //
        // Deploying class 0x0044b7358648f0c0bef25b4e36609642733275fef1b9253e21033d3d72403250 with salt 0x026b572f7817a705ae298831994ca6ab772f5ce89e57368ee677b5dae8fb34c5...
        // The contract will be deployed at address 0x044aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46
        // Contract deployment transaction: 0x02ffa6a366b7224ed8fcdc4fda5afbd3f92266e478436d6b156e060862d5440f
        // Contract deployed: 0x044aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x044aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46"),
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
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x02dfd1700117a9967ce4228c19de014ead4f5042823a95d079cf404886112d0b"),
        // The default starknet-devnet config reuses Sepolia asset addresses
        // TODO: will only work for `eth` and `strk` assets. So we will change it later on.
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
};
