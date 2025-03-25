use phf::phf_map;
use starknet_types_core::felt::Felt;

type AssetsMap = phf::Map<&'static str, Felt>;

static SEPOLIA_ASSETS_ADDRESSES: AssetsMap = phf_map! {
    "strk" => Felt::from_hex_unchecked("0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d"),
    "eth" => Felt::from_hex_unchecked("0x49D36570D4E46F48E99674BD3FCC84644DDD6B96F7C741B1562B82F9E004DC7"),
};

#[derive(Debug, Clone)]
pub struct OnChainConstants {
    pub apibara: ApibaraConstants,
    pub invoice_payment_contract_address: Felt,
    pub assets_contract_address: &'static phf::Map<&'static str, Felt>,
}

#[derive(Debug, Clone)]
pub struct ApibaraConstants {
    pub data_stream_uri: Option<&'static str>,
    pub starting_block: u64,
}

pub static ON_CHAIN_CONSTANTS: phf::Map<&'static str, OnChainConstants> = phf::phf_map! {
    "SN_SEPOLIA" =>  OnChainConstants {
        apibara: ApibaraConstants { data_stream_uri:  Some("http://sepolia.starknet.a5a.ch"), starting_block: 0 },
        invoice_payment_contract_address: Felt::ZERO, // Not deployed atm
        assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    },
    "SN_DEVNET" =>  OnChainConstants {
        apibara: ApibaraConstants { data_stream_uri: None, starting_block: 0 },
        invoice_payment_contract_address: Felt::from_hex_unchecked("0x074e3cbebe007eb4732706bec58067da01d16c0d252d763843c76612c69a4e9a"),
        assets_contract_address: &SEPOLIA_ASSETS_ADDRESSES,
    },
};
