use serde::{Deserialize, Serialize};
use primitive_types::{H160, U256};

mod assets;
pub use assets::*;
mod unit;
pub use unit::{Unit, UnitFromStrError};
mod chain_id;
pub use chain_id::ChainId;
pub mod constants;

pub const ETHEREUM_STR: &str = "ethereum";

/// Ethereum address type
pub type EthereumAddress = H160;

/// Ethereum transaction hash type
pub type EthereumTxHash = primitive_types::H256;

/// Ethereum amount type
pub type EthereumU256 = U256;

/// Validates if a string is a valid Ethereum address
pub fn is_valid_ethereum_address(address: &str) -> bool {
    if !address.starts_with("0x") {
        return false;
    }
    
    if address.len() != 42 {
        return false;
    }
    
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Converts a hex string to an Ethereum address
pub fn ethereum_address_from_hex(hex_str: &str) -> Result<EthereumAddress, hex::FromHexError> {
    let hex_str = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };
    
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 20 {
        return Err(hex::FromHexError::InvalidStringLength);
    }
    
    Ok(H160::from_slice(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ethereum_address() {
        assert!(is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"));
        assert!(is_valid_ethereum_address("0x0000000000000000000000000000000000000000"));
        assert!(!is_valid_ethereum_address("742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6")); // missing 0x
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b")); // too short
        assert!(!is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6z")); // invalid char
    }

    #[test]
    fn test_ethereum_address_from_hex() {
        let addr_str = "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6";
        let addr = ethereum_address_from_hex(addr_str).unwrap();
        assert_eq!(format!("{:?}", addr), "0x742d35cc6634c0532925a3b8d4c9db96c4b4d8b6");
    }
}
