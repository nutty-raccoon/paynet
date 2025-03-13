use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::Arc,
};

use nuts::{
    Amount,
    nut01::{self, PublicKey},
    nut02::KeysetId,
};
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
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
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

    pub async fn insert_keys<I>(&self, keyset_id: KeysetId, keys: I)
    where
        I: IntoIterator<Item = (Amount, PublicKey)>,
    {
        let mut write_lock = self.keys.write().await;
        write_lock.insert(keyset_id, keys.into_iter().collect());
    }

    pub async fn disable_keys(&self, keyset_ids: &[KeysetId]) {
        let mut write_lock = self.infos.write().await;

        for keyset_id in keyset_ids {
            if let Some(info) = write_lock.get_mut(keyset_id) {
                info.active = false;
            }
        }
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
            .map(|k| -> Result<(Amount, PublicKey), Error> {
                Ok((
                    Amount::from(k.amount),
                    PublicKey::from_str(&k.pubkey).map_err(|e| Error::Nut01(e))?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

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
    ) -> Result<(bool, Unit), Error> {
        // happy path: the infos are already in the cache
        {
            let cache_read_lock = self.infos.read().await;
            if let Some(info) = cache_read_lock.get(&keyset_id) {
                return Ok((info.active, info.unit));
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
                },
            );
        }

        Ok((db_content.active(), db_content.unit()))
    }
}
