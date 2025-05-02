use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::errors;
use dashmap::DashMap;
use node::{MeltResponse, MintResponse, SwapResponse};

/// A trait that defines a cache for storing and retrieving responses.
pub trait ResponseCache<K, V> {
    /// Retrieves a value from the cache using the specified key.
    fn get(&self, key: &K) -> Option<V>;

    /// Inserts a key-value pair into the cache.
    fn insert(&self, key: K, value: V) -> Result<(), errors::Error>;

    /// Removes a key-value pair from the cache.
    fn remove(&self, key: &K) -> bool;

    // TODO: persistent after shutting down
}

/// An in-memory implementation of the `ResponseCache` trait with optional TTL support.
#[allow(dead_code)]
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
    /// Creates a new in-memory response cache with the specified time-to-live duration.
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

/// An enum representing the different types of responses that can be cached.
#[derive(Debug, Clone)]
pub enum CachedResponse {
    /// A response from a mint operation.
    Mint(MintResponse),
    /// A response from a swap operation.
    Swap(SwapResponse),
    /// A response from a melt operation.
    Melt(MeltResponse),
}
