use phf::phf_map;
use starknet_types_core::felt::Felt;

type AssetsMap = phf::Map<&'static str, Felt>;

static SEPOLIA_ASSETS_ADDRESSES: AssetsMap = phf_map! {
    "strk" => Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"),
    "eth" => Felt::from_hex_unchecked("0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7"),
};

#[derive(Clone)]
pub struct OnChainConstants {
    pub invoice_payment_contract_address: Felt,
    pub assets_contract_address: &'static phf::Map<&'static str, Felt>,
}

pub static ONCHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf::phf_map! {
    "SN_SEPOLIA" =>  OnChainConstants {
        invoice_payment_contract_address: Felt::ZERO, // Not deployed atm
        assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    },
    "SN_DEVNET" =>  OnChainConstants {
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x0796ac33f38f759375f1392dd9299d7ce12f4f7194d7ce6aaebbd033eb48302c"),
        assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    },
};
