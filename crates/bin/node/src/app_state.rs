use std::sync::{Arc, atomic::AtomicU64};

use axum::extract::FromRef;
use nuts::{QuoteTTLConfig, nut06::NutsSettings};
use sqlx::PgPool;
use starknet_types::Unit;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::{keyset_cache::KeysetCache, methods::Method};

pub type NutsSettingsState = Arc<RwLock<NutsSettings<Method, Unit>>>;
pub type SignerClient = signer::SignerClient<Channel>;

// the application state
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pg_pool: PgPool,
    keyset_cache: KeysetCache,
    signer_client: SignerClient,
    nuts: NutsSettingsState,
    quote_ttl: Arc<QuoteTTLConfigState>,
}

impl AppState {
    pub fn new(
        pg_pool: PgPool,
        signer_client: signer::SignerClient<Channel>,
        nuts_settings: NutsSettings<Method, Unit>,
        quote_ttl: QuoteTTLConfig,
    ) -> Self {
        Self {
            pg_pool,
            keyset_cache: Default::default(),
            nuts: Arc::new(RwLock::new(nuts_settings)),
            quote_ttl: Arc::new(quote_ttl.into()),
            signer_client,
        }
    }
}

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
