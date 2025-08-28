use crate::ChainIdParseError;
use std::fmt;
use std::str::FromStr;

/// Constants representing predefined Ethereum networks
const MAINNET: &str = "1";
const SEPOLIA: &str = "11155111";
const HOLESKY: &str = "17000";
const DEVNET: &str = "1337";

/// Represents an Ethereum network identifier.
///
/// Supports standard networks (`mainnet`, `sepolia`, etc.) and
/// custom networks defined by a short string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainId {
    /// Ethereum mainnet (chain ID = 1)
    Mainnet,
    /// Ethereum Sepolia testnet (chain ID = 11155111)
    Sepolia,
    /// Ethereum Holesky testnet (chain ID = 17000)
    Holesky,
    /// Local development network (chain ID = 1337)
    Devnet,
    /// Custom chain identifier, represented as a string
    Custom(String),
}

impl ChainId {
    pub fn as_str(&self) -> &str {
        match self {
            ChainId::Mainnet => MAINNET,
            ChainId::Sepolia => SEPOLIA,
            ChainId::Holesky => HOLESKY,
            ChainId::Devnet => DEVNET,
            ChainId::Custom(s) => s,
        }
    }

    pub fn from_int(id: u64) -> Self {
        match id {
            1 => ChainId::Mainnet,
            11155111 => ChainId::Sepolia,
            17000 => ChainId::Holesky,
            1337 => ChainId::Devnet,
            _ => ChainId::Custom(id.to_string()),
        }
    }

    pub fn from_hex(hex: &str) -> Result<Self, ChainIdParseError> {
        let trimmed = hex.trim_start_matches("0x");
        let value = u64::from_str_radix(trimmed, 16)?;
        Ok(Self::from_int(value))
    }

    pub fn new_custom(s: String) -> Result<Self, ChainIdParseError> {
        if !s.is_ascii() {
            return Err(ChainIdParseError::NonAsciiCharacter);
        }
        if s.len() > 31 {
            return Err(ChainIdParseError::StringTooLong);
        }
        if s.starts_with("0x") {
            return ChainId::from_hex(&s);
        }
        Ok(Self::Custom(s))
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ChainId {
    type Err = ChainIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim().to_lowercase();
        match input.as_str() {
            "1" | "mainnet" => Ok(ChainId::Mainnet),
            "11155111" | "sepolia" => Ok(ChainId::Sepolia),
            "17000" | "holesky" => Ok(ChainId::Holesky),
            "1337" | "devnet" => Ok(ChainId::Devnet),
            _ => ChainId::new_custom(input),
        }
    }
}

impl From<u64> for ChainId {
    fn from(val: u64) -> Self {
        ChainId::from_int(val)
    }
}

impl serde::Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ChainId::from_str(&s).map_err(|_| {
            serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"a valid Ethereum chain ID (int, hex, or short string)",
            )
        })
    }
}
