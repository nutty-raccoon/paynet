use crate::Amount;
use crate::keyset_cache::CachedKeysetInfo;
use db_node::keyset::deactivate_keysets;
use grpc_service::GrpcState;
use node::{KeysetRotationService, RotateKeysetsRequest, RotateKeysetsResponse};

use std::str::FromStr;
use tonic::{Request, Response, Status};

use nuts::{nut01::PublicKey, nut02::KeysetId};
use starknet_types::Unit;

use crate::grpc_service;
#[tonic::async_trait]
impl KeysetRotationService for GrpcState {
    async fn rotate_keysets(
        &self,
        _request: Request<RotateKeysetsRequest>,
    ) -> Result<Response<RotateKeysetsResponse>, Status> {
        let mut tx = db_node::begin_db_tx(&self.pg_pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let keysets_info = db_node::keyset::get_active_keysets::<Unit>(&mut tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let mut insert_keysets_query_builder = db_node::InsertKeysetsQueryBuilder::new();

        let mut prev_keyset_ids: Vec<KeysetId> = vec![];

        // TODO: add concurency
        for (keyset_id, keyset_info) in keysets_info {
            let unit = keyset_info.unit();
            let index = keyset_info.derivation_path_index() + 1;
            let max_order = keyset_info.max_order() as u32;

            let response = self
                .signer
                .clone()
                .declare_keyset(signer::DeclareKeysetRequest {
                    unit: unit.to_string(),
                    index,
                    max_order,
                })
                .await?;

            let response = response.into_inner();

            let new_keyset_id = KeysetId::from_bytes(&response.keyset_id)
                .map_err(|e| Status::internal(e.to_string()))?;

            insert_keysets_query_builder.add_row(new_keyset_id, &unit, max_order, index);

            self.keyset_cache
                .insert_info(new_keyset_id, CachedKeysetInfo::new(true, unit))
                .await;

            self.keyset_cache
                .insert_keys(
                    new_keyset_id,
                    response
                        .keys
                        .into_iter()
                        .map(|k| {
                            (
                                Amount::from(k.amount),
                                PublicKey::from_str(&k.pubkey).unwrap(),
                            )
                        })
                        .collect(),
                )
                .await;

            prev_keyset_ids.push(keyset_id);
        }

        insert_keysets_query_builder
            .execute(&mut tx)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        deactivate_keysets(
            &mut tx,
            &prev_keyset_ids
                .iter()
                .map(|keyset_id| keyset_id.as_i64())
                .collect::<Vec<_>>(),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // disable prev ids
        self.keyset_cache.disable_keys(&prev_keyset_ids).await;

        Ok(Response::new(RotateKeysetsResponse {}))
    }
}
