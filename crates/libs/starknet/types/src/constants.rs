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

#[derive(Debug, Clone, Copy)]
pub enum AssetsAddress {
    Sepolia([(Asset, Felt); 5]),
    Devnet([(Asset, Felt); 2]),
}

impl AssetsAddress {
    pub fn get_contract_address_for_asset(&self, asset: Asset) -> Option<Felt> {
        match self {
            AssetsAddress::Sepolia(a) => a.iter(),
            AssetsAddress::Devnet(a) => a.iter(),
        }
        .find(|(a, _)| asset == *a)
        .map(|(_, address)| *address)
    }

    pub fn get_asset_for_contract_address(&self, contract_address: Felt) -> Option<Asset> {
        match self {
            AssetsAddress::Sepolia(a) => a.iter(),
            AssetsAddress::Devnet(a) => a.iter(),
        }
        .find(|(_, a)| contract_address == *a)
        .map(|(asset, _)| *asset)
    }
}

/// Assets available on Starknet Sepolia testnet with their contract addresses
///
/// These addresses are network-specific and have been verified to be the official
/// token contracts.
const SEPOLIA_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress::Sepolia([
    (
        Asset::Strk,
        Felt::from_hex_unchecked(
            "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
        ),
    ),
    (
        Asset::Eth,
        Felt::from_hex_unchecked(
            "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        ),
    ),
    (
        Asset::WBtc,
        Felt::from_hex_unchecked(
            "0x00452bd5c0512a61df7c7be8cfea5e4f893cb40e126bdc40aee6054db955129e",
        ),
    ),
    (
        Asset::UsdC,
        Felt::from_hex_unchecked(
            "0x053b40a647cedfca6ca84f542a0fe36736031905a9639a7f19a3c1e66bfd5080",
        ),
    ),
    (
        Asset::UsdT,
        Felt::from_hex_unchecked(
            "0x02ab8758891e84b968ff11361789070c6b1af2df618d6d2f4a78b0757573c6eb",
        ),
    ),
]);

/// Devnet assets with placeholder addresses
const DEVNET_ASSETS_ADDRESSES: AssetsAddress = AssetsAddress::Devnet([
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
    pub invoice_payment_contract_address: Felt,
    pub assets_contract_address: AssetsAddress,
}

/// Map of all supported networks and their corresponding constants
///
/// This is the primary entry point for accessing network-specific configuration.
/// New networks can be added here without modifying the rest of the codebase.
pub static ON_CHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf::phf_map! {
    "SN_SEPOLIA" =>  OnChainConstants {
        // Starting block is the one which contains the invoice_payment_contract deployment
        // Tx: 0x3ff1f5d34e471b30f12bd28f69c4edfc25c40856b8ca269d92bc1fe1bd3da11
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x03b7d6935858cc0e84cba7267cc9daa76dfaf060303761608f12cf84191e3571"),
        assets_contract_address: SEPOLIA_ASSETS_ADDRESSES,
    },
    "SN_DEVNET" =>  OnChainConstants {
        // This address is guaranted to be correct, if and only if,
        // you are using our `starknet-on-chain-setup` rust deployment executable.
        // It is automaticaly used when setting up the network using this repo's `docker-compose.yml`
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x054eb8613832317fc641555b852b0a3b4cef5cc444fccab5e3de94430fb8fcda"),
        // The default starknet-devnet config reuses Sepolia asset addresses
        // TODO: will only work for `eth` and `strk` assets. So we will change it later on.
        assets_contract_address: DEVNET_ASSETS_ADDRESSES,
    },
};
