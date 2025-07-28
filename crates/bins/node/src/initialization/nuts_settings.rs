use nuts::{Amount, nut04::MintMethodSettings, nut05::MeltMethodSettings, nut06::NutsSettings};

use crate::methods::Method;

// Define a unified unit enum that can represent units from different chains
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum UnifiedUnit {
    #[cfg(feature = "starknet")]
    MilliStrk,
    #[cfg(feature = "starknet")]
    Gwei,
    #[cfg(feature = "ethereum")]
    MilliUsdc,
    #[cfg(feature = "ethereum")]
    EthGwei,
}

impl std::fmt::Display for UnifiedUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "starknet")]
            UnifiedUnit::MilliStrk => write!(f, "millistrk"),
            #[cfg(feature = "starknet")]
            UnifiedUnit::Gwei => write!(f, "gwei"),
            #[cfg(feature = "ethereum")]
            UnifiedUnit::MilliUsdc => write!(f, "milliusdc"),
            #[cfg(feature = "ethereum")]
            UnifiedUnit::EthGwei => write!(f, "gwei"),
        }
    }
}

impl AsRef<str> for UnifiedUnit {
    fn as_ref(&self) -> &str {
        match self {
            #[cfg(feature = "starknet")]
            UnifiedUnit::MilliStrk => "millistrk",
            #[cfg(feature = "starknet")]
            UnifiedUnit::Gwei => "gwei",
            #[cfg(feature = "ethereum")]
            UnifiedUnit::MilliUsdc => "milliusdc",
            #[cfg(feature = "ethereum")]
            UnifiedUnit::EthGwei => "gwei",
        }
    }
}

impl From<UnifiedUnit> for u32 {
    fn from(unit: UnifiedUnit) -> Self {
        match unit {
            #[cfg(feature = "starknet")]
            UnifiedUnit::MilliStrk => 0,
            #[cfg(feature = "starknet")]
            UnifiedUnit::Gwei => 1,
            #[cfg(feature = "ethereum")]
            UnifiedUnit::MilliUsdc => 2,
            #[cfg(feature = "ethereum")]
            UnifiedUnit::EthGwei => 3,
        }
    }
}

impl std::str::FromStr for UnifiedUnit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "starknet")]
            "millistrk" => Ok(UnifiedUnit::MilliStrk),
            #[cfg(feature = "starknet")]
            "gwei" => Ok(UnifiedUnit::Gwei),
            #[cfg(feature = "ethereum")]
            "milliusdc" => Ok(UnifiedUnit::MilliUsdc),
            #[cfg(feature = "ethereum")]
            "gwei" if cfg!(feature = "ethereum") => Ok(UnifiedUnit::EthGwei),
            _ => Err(format!("Unknown unit: {}", s)),
        }
    }
}

impl nuts::traits::Unit for UnifiedUnit {}

// TODO: make it a compile time const
pub(super) fn nuts_settings() -> NutsSettings<Method, UnifiedUnit> {
    let mut mint_methods = Vec::new();
    let mut melt_methods = Vec::new();

    #[cfg(feature = "starknet")]
    {
        mint_methods.push(MintMethodSettings {
            method: Method::Starknet,
            unit: UnifiedUnit::MilliStrk,
            min_amount: Some(Amount::ONE),
            max_amount: None,
            description: true,
        });
        melt_methods.push(MeltMethodSettings {
            method: Method::Starknet,
            unit: UnifiedUnit::MilliStrk,
            min_amount: Some(Amount::ONE),
            max_amount: None,
        });
    }

    #[cfg(feature = "ethereum")]
    {
        mint_methods.push(MintMethodSettings {
            method: Method::Ethereum,
            unit: UnifiedUnit::MilliUsdc,
            min_amount: Some(Amount::ONE),
            max_amount: None,
            description: true,
        });
        melt_methods.push(MeltMethodSettings {
            method: Method::Ethereum,
            unit: UnifiedUnit::MilliUsdc,
            min_amount: Some(Amount::ONE),
            max_amount: None,
        });
    }

    NutsSettings {
        nut04: nuts::nut04::Settings {
            methods: mint_methods,
            disabled: false,
        },
        nut05: nuts::nut05::Settings {
            methods: melt_methods,
            disabled: false,
        },
        nut09: nuts::nut06::SupportedSettings { supported: true },
        nut19: nuts::nut19::Settings { ttl: None },
    }
}
