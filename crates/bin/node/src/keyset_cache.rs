use std::{collections::HashMap, sync::Arc};

use nuts::nut02::KeysetId;
use sqlx::PgConnection;
use starknet_types::Unit;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to load keyset with id {0} in db: {1}")]
    UnknownKeysetId(KeysetId, #[source] db_node::Error),
}

#[derive(Debug, Clone)]
pub struct CachedKeysetInfo {
    active: bool,
    unit: Unit,
}

impl CachedKeysetInfo {
    pub fn new(active: bool, unit: Unit) -> Self {
        Self { active, unit }
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn unit(&self) -> Unit {
        self.unit
    }
}

impl From<db_node::KeysetInfo<Unit>> for CachedKeysetInfo {
    fn from(value: db_node::KeysetInfo<Unit>) -> Self {
        Self {
            active: value.active(),
            unit: value.unit(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeysetCache {
    keysets: Arc<RwLock<HashMap<KeysetId, CachedKeysetInfo>>>,
}

impl KeysetCache {
    pub async fn insert(&self, keyset_id: KeysetId, info: CachedKeysetInfo) {
        let mut write_lock = self.keysets.write().await;

        write_lock.insert(keyset_id, info);
    }

    pub async fn get_keyset_info(
        &self,
        conn: &mut PgConnection,
        keyset_id: KeysetId,
    ) -> Result<CachedKeysetInfo, Error> {
        // happy path: the infos are already in the cache
        {
            let cache_read_lock = self.keysets.read().await;
            if let Some(info) = cache_read_lock.get(&keyset_id) {
                return Ok(info.clone());
            }
        }

        // Load the infos from db
        let keyset_info: CachedKeysetInfo = db_node::get_keyset::<Unit>(conn, &keyset_id)
            .await
            .map_err(|e| Error::UnknownKeysetId(keyset_id, e))?
            .into();

        // Save the infos in the cache
        {
            let mut cache_write_lock = self.keysets.write().await;
            cache_write_lock.insert(keyset_id, keyset_info.clone());
        }

        Ok(keyset_info)
    }
}
