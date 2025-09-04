use bitcoin::bip32::Xpriv;
use futures::{StreamExt, future::join_all};
use node_client::{CheckStateRequest, GetKeysetsRequest, NodeClient, RestoreRequest};
use nuts::{
    Amount,
    dhke::{self, hash_to_curve},
    nut01::{self, PublicKey},
    nut02::KeysetId,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;
use tracing::error;

use crate::{
    ConnectToNodeError, StoreNewProofsError,
    db::{self, keyset},
    seed_phrase, store_new_proofs_from_blind_signatures,
    types::NodeUrl,
    wallet::SeedPhraseManager,
};

#[derive(Debug, thiserror::Error)]
pub enum RegisterNodeError {
    #[error("failed connect to the node: {0}")]
    Connect(#[from] ConnectToNodeError),
    #[error("failed connect to database: {0}")]
    R2d2(#[from] r2d2::Error),
    #[error("unknown node with url: {0}")]
    NotFound(NodeUrl),
    #[error("fail to interact with the database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("fail to refresh the node {0} keyset: {1}")]
    RefreshNodeKeyset(u32, RefreshNodeKeysetError),
}

pub async fn register(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_url: &NodeUrl,
) -> Result<u32, RegisterNodeError> {
    let node_id = {
        let db_conn = pool.get()?;
        db::node::insert(&db_conn, node_url)?;
        db::node::get_id_by_url(&db_conn, node_url)?
            .ok_or(RegisterNodeError::NotFound(node_url.clone()))?
    };

    refresh_keysets(pool, node_client, node_id)
        .await
        .map_err(|e| RegisterNodeError::RefreshNodeKeyset(node_id, e))?;

    Ok(node_id)
}

#[derive(Debug, thiserror::Error)]
pub enum RestoreNodeError {
    #[error(transparent)]
    R2d2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
    #[error(transparent)]
    Client(#[from] tonic::Status),
    #[error(transparent)]
    StoreNewTokens(#[from] StoreNewProofsError),
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
    #[error(transparent)]
    Dhke(#[from] dhke::Error),
    #[error("`restore` restponse contains an output that was not part of the query")]
    UnknownBlindSecretInRestoreResponse,
    #[error("failed to interact with wallet")]
    Wallet(#[from] crate::wallet::Error),
}

pub async fn restore(
    seed_phrase_manager: impl SeedPhraseManager,
    pool: Pool<SqliteConnectionManager>,
    node_id: u32,
    node_client: NodeClient<Channel>,
) -> Result<(), RestoreNodeError> {
    let keyset_ids = {
        let db_conn = pool.get()?;
        keyset::get_all_ids_for_node(&db_conn, node_id)?
    };

    let xpriv = crate::wallet::get_private_key(seed_phrase_manager)?;
    let mut handles = Vec::with_capacity(keyset_ids.len());
    for keyset_id in keyset_ids {
        handles.push(restore_keyset(
            pool.clone(),
            node_id,
            node_client.clone(),
            xpriv,
            keyset_id,
        ));
    }
    let results = join_all(handles).await;
    for res in results {
        res?;
    }

    Ok(())
}

async fn restore_keyset(
    pool: Pool<SqliteConnectionManager>,
    node_id: u32,
    mut node_client: NodeClient<Channel>,
    xpriv: Xpriv,
    keyset_id: KeysetId,
) -> Result<(), RestoreNodeError> {
    let mut empty_response_counter = 0;
    let mut n_batch_done = 0;

    while empty_response_counter < 3 {
        let start_count = n_batch_done * 100;
        let (blinded_messages, secrets) = seed_phrase::generate_blinded_messages(
            keyset_id,
            xpriv,
            start_count,
            start_count + 99,
        )?;

        let outputs = blinded_messages
            .iter()
            .map(|bm| node_client::BlindedMessage {
                amount: bm.amount.into(),
                keyset_id: bm.keyset_id.to_bytes().to_vec(),
                blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
            })
            .collect();

        let request = RestoreRequest { outputs };

        let response = node_client::NodeClient::restore(&mut node_client, request)
            .await?
            .into_inner();

        if response.signatures.is_empty() {
            empty_response_counter += 1;
        } else {
            // Get the index of the last blind secret known byt the node
            // The node restore impl is guaranteed to return all values in order, but not to return all values.
            // It will only return the ones he has seen in the past, preserving relative ordering.
            // Which mean that the index of a value in the node response is not guaranteed
            // to be equal to it index in the query (and therefore its counter value).
            // This is only true if the node return as many values as there was in the query, then it is 100.
            // Otherwise, we have to iterate over the query values to find the one matching the last one return by the node,
            // and use its index as counter.
            let counter_last_known_blinded_secret = start_count
                + if response.outputs.len() == 100 {
                    99
                } else {
                    let last_bs =
                        PublicKey::from_slice(&response.outputs.last().unwrap().blinded_secret)?;
                    blinded_messages
                        .iter()
                        .position(|bm| bm.blinded_secret == last_bs)
                        .ok_or(RestoreNodeError::UnknownBlindSecretInRestoreResponse)?
                        as u32
                };

            let ys = response
                .outputs
                .iter()
                .map(|o| -> Result<Vec<u8>, RestoreNodeError> {
                    let blinded_secret = PublicKey::from_slice(&o.blinded_secret)?;
                    let (secret, _r) = secrets[&blinded_secret].clone();
                    let y: PublicKey = hash_to_curve(&secret.to_bytes())?;

                    Ok(y.to_bytes().to_vec())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let check_state_response = node_client
                .check_state(CheckStateRequest { ys })
                .await?
                .into_inner();

            let iterator = response
                .outputs
                .into_iter()
                .zip(response.signatures)
                .zip(check_state_response.states)
                .filter_map(|((bm, bs), ps)| -> Option<Result<_, nut01::Error>> {
                    if ps.state() != node_client::ProofState::PsUnspent {
                        None
                    } else {
                        let blind_signature = match PublicKey::from_slice(&bs.blind_signature) {
                            Ok(bs) => bs,
                            Err(e) => return Some(Err(e)),
                        };
                        let blinded_secret = match PublicKey::from_slice(&bm.blinded_secret) {
                            Ok(bs) => bs,
                            Err(e) => return Some(Err(e)),
                        };
                        let (secret, r) = secrets[&blinded_secret].clone();

                        Some(Ok((blind_signature, secret, r, Amount::from(bs.amount))))
                    }
                });

            let mut db_conn = pool.get()?;
            let tx = db_conn.transaction()?;
            store_new_proofs_from_blind_signatures(&tx, node_id, keyset_id, iterator)?;
            db::keyset::set_counter(&tx, keyset_id, counter_last_known_blinded_secret + 1)?;
            tx.commit()?;
        }
        n_batch_done += 1;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshNodeKeysetError {
    #[error("failed to get keysets from the node: {0}")]
    GetKeysets(#[from] tonic::Status),
    #[error("failed connect to database: {0}")]
    R2d2(#[from] r2d2::Error),
    #[error("fail to interact with the database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("conversion error: {0}")]
    InvalidKeysetValue(String),
}

pub async fn refresh_keysets(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
) -> Result<(), RefreshNodeKeysetError> {
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;

    let new_keyset_ids = {
        let db_conn = pool.get()?;
        crate::db::keyset::upsert_many_for_node(&db_conn, node_id, keysets)?
    };

    // Parallelization of the queries
    let mut futures = futures::stream::FuturesUnordered::new();
    for new_keyset_id in new_keyset_ids {
        let mut cloned_node_client = node_client.clone();
        futures.push(async move {
            cloned_node_client
                .keys(node_client::GetKeysRequest {
                    keyset_id: Some(new_keyset_id.to_bytes().to_vec()),
                })
                .await
        })
    }

    while let Some(res) = futures.next().await {
        match res {
            // Save the keys in db
            Ok(resp) => {
                let resp = resp.into_inner();
                let keyset = resp.keysets;
                let id = KeysetId::from_bytes(&keyset[0].id).map_err(|e| {
                    RefreshNodeKeysetError::InvalidKeysetValue(format!(
                        "Invalid keyset ID length: {:?}",
                        e
                    ))
                })?;
                let db_conn = pool.get()?;
                db::insert_keyset_keys(
                    &db_conn,
                    id,
                    keyset[0].keys.iter().map(|k| (k.amount, k.pubkey.as_str())),
                )?;
            }
            Err(e) => {
                error!("could not get keys for one of the keysets: {}", e);
            }
        }
    }

    Ok(())
}
