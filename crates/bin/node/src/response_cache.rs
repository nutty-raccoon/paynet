use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::errors;
use dashmap::DashMap;
use node::{MeltResponse, MintQuoteResponse, MintResponse, SwapResponse};

pub trait ResponseCache<K, V> {
    // Basic operations
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V) -> Result<(), errors::Error>;
    fn remove(&self, key: &K) -> bool;
}

#[derive(Debug)]
pub struct InMemResponseCache<K, V>
where
    K: Eq + std::hash::Hash + Debug,
    V: Clone,
{
    store: DashMap<K, (V, Instant)>,
    ttl: Option<Duration>,
}

impl<K, V> InMemResponseCache<K, V>
where
    K: Eq + std::hash::Hash + Debug,
    V: Clone,
{
    pub fn new(ttl: Option<Duration>) -> Self {
        Self {
            store: DashMap::new(),
            ttl,
        }
    }
}

impl<K, V> ResponseCache<K, V> for InMemResponseCache<K, V>
where
    K: Eq + std::hash::Hash + Debug,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        let entry = self.store.get(key)?;
        let (value, _created_at) = &*entry;
        Some(value.clone())
    }

    fn insert(&self, key: K, value: V) -> Result<(), errors::Error> {
        self.store.insert(key, (value, Instant::now()));
        Ok(())
    }

    fn remove(&self, key: &K) -> bool {
        self.store.remove(key).is_some()
    }
}

#[derive(Debug, Clone)]
pub enum CachedResponse {
    Mint(MintResponse),
    Melt(MeltResponse),
    Swap(SwapResponse),
    MintQuote(MintQuoteResponse),
}
