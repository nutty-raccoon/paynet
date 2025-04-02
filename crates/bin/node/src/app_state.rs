use std::sync::{Arc, atomic::AtomicU64};

use nuts::{QuoteTTLConfig, nut06::NutsSettings};
use starknet_types::Unit;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::methods::Method;

pub type NutsSettingsState = Arc<RwLock<NutsSettings<Method, Unit>>>;
pub type SignerClient = signer::SignerClient<Channel>;

/// Quote Time To Live config
///
/// Specifies for how long, in seconds, the quote issued by the node will be valid.
///
/// We use AtomicU64 to share this easily between threads.
#[derive(Debug)]
pub struct QuoteTTLConfigState {
    mint_ttl: AtomicU64,
    melt_ttl: AtomicU64,
}

impl QuoteTTLConfigState {
    /// Returns the number of seconds a new mint quote is valid for
    pub fn mint_ttl(&self) -> u64 {
        self.mint_ttl.load(std::sync::atomic::Ordering::Relaxed)
    }
    /// Returns the number of seconds a new melt quote is valid for
    pub fn melt_ttl(&self) -> u64 {
        self.melt_ttl.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl From<QuoteTTLConfig> for QuoteTTLConfigState {
    fn from(value: QuoteTTLConfig) -> Self {
        Self {
            mint_ttl: value.mint_ttl.into(),
            melt_ttl: value.melt_ttl.into(),
        }
    }
}

#[cfg(feature = "starknet")]
pub mod starknet {
    use liquidity_source::starknet::{StarknetDepositer, StarknetWithdrawer};

    #[derive(Debug, Clone)]
    pub struct StarknetConfig {
        pub withdrawer: StarknetWithdrawer,
        pub depositer: StarknetDepositer,
    }
}
