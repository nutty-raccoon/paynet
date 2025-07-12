use node::{Key, KeysetKeys};
use nuts::nut02::KeysetId;
use sqlx::PgConnection;
use tonic::Status;

use crate::grpc_service::GrpcState;

impl GrpcState {
    pub async fn inner_keys_for_keyset_id(
        &self,
        db_conn: &mut PgConnection,
        keyset_id: Vec<u8>,
    ) -> Result<Vec<KeysetKeys>, Status> {
        let keyset_id = KeysetId::from_bytes(&keyset_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let keys = self
            .keyset_cache
            .get_keyset_keys(db_conn, self.signer.clone(), keyset_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let keyset_info = self
            .keyset_cache
            .get_keyset_info(db_conn, keyset_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(vec![KeysetKeys {
            id: keyset_id.to_bytes().to_vec(),
            unit: keyset_info.unit().to_string(),
            active: keyset_info.active(),
            keys: keys
                .into_iter()
                .map(|(a, pk)| Key {
                    amount: a.into(),
                    pubkey: pk.to_string(),
                })
                .collect(),
        }])
    }

    pub async fn inner_keys_no_keyset_id(
        &self,
        db_conn: &mut PgConnection,
    ) -> Result<Vec<KeysetKeys>, Status> {
        let keysets_info = db_node::keyset::get_active_keysets::<String>(db_conn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut keysets = Vec::with_capacity(keysets_info.len());
        // TODO: add concurency
        for (keyset_id, keyset_info) in keysets_info {
            let keys = self
                .keyset_cache
                .get_keyset_keys(db_conn, self.signer.clone(), keyset_id)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            keysets.push(KeysetKeys {
                id: keyset_id.to_bytes().to_vec(),
                unit: keyset_info.unit(),
                active: keyset_info.active(),
                keys: keys
                    .into_iter()
                    .map(|(a, pk)| Key {
                        amount: a.into(),
                        pubkey: pk.to_string(),
                    })
                    .collect(),
            })
        }

        Ok(keysets)
    }
}
