use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

use crate::errors;
use dashmap::DashMap;
use node::{MeltResponse, MintResponse, SwapResponse};

pub trait ResponseCache<K, V> {
    // Basic operations
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V) -> Result<(), errors::Error>;
    #[allow(dead_code)]
    fn remove(&self, key: &K) -> bool;

    #[allow(dead_code)]
    // For cleanup/shutdown
    fn clear(&self);
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
        let (value, created_at) = &*entry;

        if let Some(ttl) = self.ttl {
            if created_at.elapsed() > ttl {
                drop(entry);
                self.store.remove(key);
                return None;
            }
        }

        Some(value.clone())
    }

    fn insert(&self, key: K, value: V) -> Result<(), errors::Error> {
        self.store.insert(key, (value, Instant::now()));
        Ok(())
    }

    fn remove(&self, key: &K) -> bool {
        self.store.remove(key).is_some()
    }

    fn clear(&self) {
        self.store.clear();
    }
}

#[derive(Debug, Clone)]
pub enum CachedResponse {
    Mint(MintResponse),
    Melt(MeltResponse),
    Swap(SwapResponse),
}
