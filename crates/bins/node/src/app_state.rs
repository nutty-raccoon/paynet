use std::sync::{Arc, atomic::AtomicU64};

use nuts::{QuoteTTLConfig, nut06::NutsSettings};
use serde::Serialize;
use sqlx::PgPool;
use starknet_types::Unit;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::{
    keyset_cache::KeysetCache,
    liquidity_sources::LiquiditySources,
    methods::Method,
    response_cache::{CachedResponse, InMemResponseCache},
};
use nuts::nut19::Route;

pub type NutsSettingsState = Arc<RwLock<NutsSettings<Method, Unit>>>;
pub type SignerClient = signer::SignerClient<tower_otel::trace::Grpc<Channel>>;

#[derive(Debug, Clone, Serialize)]
pub struct Key {
    pub amount: u64,
    pub pubkey: String,
}

impl Into<node::Key> for Key {
    fn into(self) -> node::Key {
        node::Key {
            amount: self.amount,
            pubkey: self.pubkey,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KeysetKeys {
    pub id: Vec<u8>,
    pub unit: String,
    pub active: bool,
    pub keys: Vec<Key>,
}

impl Into<node::KeysetKeys> for KeysetKeys {
    fn into(self) -> node::KeysetKeys {
        node::KeysetKeys {
            id: self.id,
            unit: self.unit,
            active: self.active,
            keys: self.keys.into_iter().map(Into::into).collect(),
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

/// Shared application state used by both gRPC and HTTP services
#[derive(Debug, Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
    pub signer: SignerClient,
    pub keyset_cache: KeysetCache,
    pub nuts: NutsSettingsState,
    pub quote_ttl: Arc<QuoteTTLConfigState>,
    pub liquidity_sources: LiquiditySources<Unit>,
    pub response_cache: Arc<InMemResponseCache<(Route, u64), CachedResponse>>,
}

impl AppState {
    pub fn new(
        pg_pool: PgPool,
        signer_client: SignerClient,
        nuts_settings: NutsSettings<Method, Unit>,
        quote_ttl: QuoteTTLConfig,
        liquidity_sources: LiquiditySources<Unit>,
    ) -> Self {
        Self {
            pg_pool,
            keyset_cache: Default::default(),
            nuts: Arc::new(RwLock::new(nuts_settings)),
            quote_ttl: Arc::new(quote_ttl.into()),
            signer: signer_client,
            liquidity_sources,
            response_cache: Arc::new(InMemResponseCache::new(None)),
        }
    }
}
