use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::Arc,
};

use nuts::{Amount, nut01::PublicKey, nut02::KeysetId};
use sqlx::PgConnection;
use starknet_types::Unit;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::app_state::SignerClient;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to load keyset with id {0} in db: {1}")]
    UnknownKeysetId(KeysetId, #[source] db_node::Error),
    #[error(transparent)]
    SignerClient(#[from] tonic::Status),
}

#[derive(Debug, Clone)]
pub struct CachedKeysetInfo {
    active: bool,
    unit: Unit,
    max_order: u32,
}

impl CachedKeysetInfo {
    pub fn new(active: bool, unit: Unit, max_order: u32) -> Self {
        Self { active, unit, max_order }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeysetCache {
    infos: Arc<RwLock<HashMap<KeysetId, CachedKeysetInfo>>>,
    keys: Arc<RwLock<HashMap<KeysetId, BTreeMap<Amount, PublicKey>>>>,
}

impl KeysetCache {
    pub async fn insert_info(&self, keyset_id: KeysetId, info: CachedKeysetInfo) {
        let mut write_lock = self.infos.write().await;

        write_lock.insert(keyset_id, info);
    }

    pub async fn insert_keys(&self, keyset_id: KeysetId, keys: BTreeMap<Amount, PublicKey>) {
        let mut write_lock = self.keys.write().await;

        write_lock.insert(keyset_id, keys);
    }

    pub async fn get_keyset_keys(
        &self,
        conn: &mut PgConnection,
        signer: SignerClient,
        keyset_id: KeysetId,
    ) -> Result<BTreeMap<Amount, PublicKey>, Error> {
        // happy path: the infos are already in the cache
        {
            let cache_read_lock = self.keys.read().await;
            if let Some(info) = cache_read_lock.get(&keyset_id) {
                return Ok(info.clone());
            }
        }

        // Load the infos from db
        let db_content = db_node::keyset::get_keyset::<Unit>(conn, &keyset_id)
            .await
            .map_err(|e| Error::UnknownKeysetId(keyset_id, e))?;

        let signer_response = signer
            .clone()
            .declare_keyset(signer::DeclareKeysetRequest {
                unit: db_content.unit().to_string(),
                index: db_content.derivation_path_index(),
                max_order: db_content.max_order().into(),
            })
            .await?;
        let signer_keyset_info = signer_response.into_inner();
        let keys = signer_keyset_info
            .keys
            .into_iter()
            .map(|k| {
                (
                    Amount::from(k.amount),
                    PublicKey::from_str(&k.pubkey).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        // Save the infos in the cache
        {
            let mut cache_write_lock = self.keys.write().await;
            cache_write_lock.insert(keyset_id, keys.clone());
        }

        Ok(keys)
    }

    pub async fn get_keyset_info(
        &self,
        conn: &mut PgConnection,
        keyset_id: KeysetId,
    ) -> Result<(bool, Unit, u32), Error> {
        // happy path: the infos are already in the cache
        {
            let cache_read_lock = self.infos.read().await;
            if let Some(info) = cache_read_lock.get(&keyset_id) {
                return Ok((info.active, info.unit, info.max_order));
            }
        }

        // Load the infos from db
        let db_content = db_node::keyset::get_keyset::<Unit>(conn, &keyset_id)
            .await
            .map_err(|e| Error::UnknownKeysetId(keyset_id, e))?;

        // Save the infos in the cache
        {
            let mut cache_write_lock = self.infos.write().await;
            cache_write_lock.insert(
                keyset_id,
                CachedKeysetInfo {
                    active: db_content.active(),
                    unit: db_content.unit(),
                    max_order: db_content.max_order().into(),
                },
            );
        }

        Ok((db_content.active(), db_content.unit(), db_content.max_order().into()))
    }
}
