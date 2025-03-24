use crate::CairoShortStringToFeltError;

const SN_MAINNET: &str = "SN_MAINNET";
const SN_SEPOLIA: &str = "SN_SEPOLIA";
const SN_DEVNET: &str = "SN_DEVNET";

/// The chain where the represented assets live
#[derive(Debug, Clone)]
pub enum ChainId {
    /// Starknet mainnet
    Mainnet,
    /// Starknet sepolia testnet
    Sepolia,
    /// Starknet local devnet
    Devnet,
    /// A custom network
    ///
    /// The inner value should be a valid cairo short string, otherwise IO will panic
    Custom(String),
}

impl ChainId {
    pub fn new_custom(s: String) -> Result<Self, CairoShortStringToFeltError> {
        if !s.is_ascii() {
            return Err(CairoShortStringToFeltError::NonAsciiCharacter);
        }
        if s.len() > 31 {
            return Err(CairoShortStringToFeltError::StringTooLong);
        }

        Ok(Self::Custom(s))
    }
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::Mainnet => std::fmt::Display::fmt(SN_MAINNET, f),
            ChainId::Sepolia => std::fmt::Display::fmt(SN_SEPOLIA, f),
            ChainId::Devnet => std::fmt::Display::fmt(SN_DEVNET, f),
            ChainId::Custom(chain_id) => std::fmt::Display::fmt(&chain_id, f),
        }
    }
}

impl AsRef<str> for ChainId {
    fn as_ref(&self) -> &str {
        match self {
            ChainId::Mainnet => SN_MAINNET,
            ChainId::Sepolia => SN_SEPOLIA,
            ChainId::Devnet => SN_DEVNET,
            ChainId::Custom(s) => s,
        }
    }
}

impl serde::Serialize for ChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let as_string = self.to_string();

        serializer.serialize_str(&as_string)
    }
}

impl<'de> serde::Deserialize<'de> for ChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let short_string = <String>::deserialize(deserializer)?;
        match short_string.as_str() {
            SN_MAINNET => Ok(ChainId::Mainnet),
            SN_SEPOLIA => Ok(ChainId::Sepolia),
            SN_DEVNET => Ok(ChainId::Devnet),
            s => ChainId::new_custom(s.to_string()).map_err(|_| {
                serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(s),
                    &"a valid cairo short string",
                )
            }),
        }
    }
}
