pub mod db;
pub mod errors;
pub mod melt;
pub mod mint;
mod outputs;
pub mod types;

use std::str::FromStr;

use errors::Error;
use futures::StreamExt;
use itertools::Itertools;
use node_client::{AcknowledgeRequest, GetKeysetsRequest, NodeClient, hash_swap_request};
use num_traits::{CheckedAdd, Zero};
use nuts::dhke::{hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut00::{self, BlindedMessage, Proof};
use nuts::nut01::{PublicKey, SecretKey};
use nuts::nut02::KeysetId;
use nuts::nut19::Route;
use nuts::traits::Unit;
use nuts::{Amount, SplitTarget};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, params};
use tonic::Request;
use tonic::transport::Channel;
use types::compact_wad::{CompactKeysetProofs, CompactProof, CompactWad};
use types::{NodeUrl, PreMint, ProofState};

pub fn convert_inputs(inputs: &[Proof]) -> Vec<node_client::Proof> {
    inputs
        .iter()
        .map(|p| node_client::Proof {
            amount: p.amount.into(),
            keyset_id: p.keyset_id.to_bytes().to_vec(),
            secret: p.secret.to_string(),
            unblind_signature: p.c.to_bytes().to_vec(),
        })
        .collect()
}

pub fn convert_outputs(outputs: &[BlindedMessage]) -> Vec<node_client::BlindedMessage> {
    outputs
        .iter()
        .map(|o| node_client::BlindedMessage {
            amount: o.amount.into(),
            keyset_id: o.keyset_id.to_bytes().to_vec(),
            blinded_secret: o.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub fn build_outputs_from_premints(
    keyset_id: [u8; 8],
    pre_mints: &[PreMint],
) -> Vec<node_client::BlindedMessage> {
    pre_mints
        .iter()
        .map(|pm| node_client::BlindedMessage {
            amount: pm.amount.into(),
            keyset_id: keyset_id.to_vec(),
            blinded_secret: pm.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub async fn refresh_node_keysets(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
) -> Result<(), Error> {
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;

    let new_keyset_ids = {
        let db_conn = pool.get()?;
        crate::db::upsert_node_keysets(&db_conn, node_id, keysets)?
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
                let id = KeysetId::from_bytes(&keyset[0].id)
                    .map_err(|e| Error::Conversion(format!("Invalid keyset ID length: {:?}", e)))?;
                let db_conn = pool.get()?;
                db::insert_keyset_keys(
                    &db_conn,
                    id,
                    keyset[0].keys.iter().map(|k| (k.amount, k.pubkey.as_str())),
                )?;
            }
            Err(e) => {
                log::error!("could not get keys for one of the keysets: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn read_or_import_node_keyset(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    keyset_id: KeysetId,
) -> Result<(String, u64), Error> {
    // Happy path, it is in DB
    {
        let db_conn = pool.get()?;
        if let Some(unit) = db::get_keyset_unit(&db_conn, keyset_id)? {
            // Should be safe to unwrap unless someone manually tamper with the database to remove keys
            let max_order = db::proof::get_max_order_for_keyset(&db_conn, keyset_id)?.unwrap();
            return Ok((unit, max_order));
        }
    }

    let keyset_id_as_bytes = keyset_id.to_bytes();

    let resp = node_client
        .keys(node_client::GetKeysRequest {
            keyset_id: Some(keyset_id_as_bytes.to_vec()),
        })
        .await?
        .into_inner();
    let keyset = resp.keysets.first().unwrap();
    let max_order = keyset.keys.iter().map(|k| k.amount).max().unwrap();

    let db_conn = pool.get()?;
    db_conn.execute(
        "INSERT INTO keyset (id, node_id, unit, active) VALUES (?1, ?2, ?3, ?4)",
        params![keyset_id_as_bytes, node_id, &keyset.unit, keyset.active],
    )?;

    db::insert_keyset_keys(
        &db_conn,
        keyset_id,
        keyset.keys.iter().map(|k| (k.amount, k.pubkey.as_str())),
    )?;

    Ok((keyset.unit.clone(), max_order))
}

pub fn get_active_keyset_for_unit(
    db_conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<KeysetId, Error> {
    let keyset_id = db::fetch_one_active_keyset_id_for_node_and_unit(db_conn, node_id, unit)?
        .ok_or(Error::NoMatchingKeyset)?;

    Ok(keyset_id)
}

pub fn store_new_tokens(
    db_conn: &Connection,
    node_id: u32,
    keyset_id: KeysetId,
    pre_mints: impl Iterator<Item = PreMint>,
    signatures: impl Iterator<Item = node_client::BlindSignature>,
) -> Result<Vec<(PublicKey, Amount)>, Error> {
    let signatures_iterator = signatures
        .into_iter()
        .map(|bs| PublicKey::from_slice(&bs.blind_signature))
        .collect::<Result<Vec<_>, _>>()?;

    let signatures_iterator = pre_mints
        .into_iter()
        .zip(signatures_iterator)
        .map(|(pm, signature)| (signature, pm.secret, pm.r, pm.amount));

    store_new_proofs_from_blind_signatures(db_conn, node_id, keyset_id, signatures_iterator)
}

pub fn store_new_proofs_from_blind_signatures(
    db_conn: &Connection,
    node_id: u32,
    keyset_id: KeysetId,
    signatures_iterator: impl IntoIterator<Item = (PublicKey, Secret, SecretKey, Amount)>,
) -> Result<Vec<(PublicKey, Amount)>, Error> {
    const GET_PUBKEY: &str = r#"
        SELECT pubkey FROM key WHERE keyset_id = ?1 and amount = ?2 LIMIT 1;
    "#;
    const INSERT_PROOF: &str = r#"
        INSERT INTO proof
            (y, node_id, keyset_id, amount, secret, unblind_signature, state)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    "#;
    let mut get_pubkey_stmt = db_conn.prepare(GET_PUBKEY)?;
    let mut insert_proof_stmt = db_conn.prepare(INSERT_PROOF)?;

    let mut new_tokens = Vec::new();

    for (blinded_message, secret, r, amount) in signatures_iterator {
        let node_key_pubkey = PublicKey::from_str(
            &get_pubkey_stmt
                .query_row(params![keyset_id, amount], |row| row.get::<_, String>(0))?,
        )?;
        let unblinded_signature: PublicKey =
            unblind_message(&blinded_message, &r, &node_key_pubkey)?;

        let y = hash_to_curve(secret.as_ref())?;

        insert_proof_stmt.execute(params![
            &y,
            node_id,
            keyset_id,
            amount,
            secret,
            &unblinded_signature,
            ProofState::Unspent,
        ])?;

        new_tokens.push((y, amount));
    }

    Ok(new_tokens)
}

pub async fn fetch_inputs_ids_from_db_or_node(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    target_amount: Amount,
    unit: &str,
) -> Result<Option<Vec<PublicKey>>, Error> {
    let mut proofs_ids = Vec::new();
    let mut proofs_not_used = Vec::new();
    let mut remaining_amount = target_amount;

    {
        let db_conn = pool.get()?;
        let total_amount_available =
            db::proof::compute_total_amount_of_available_proofs(&db_conn, node_id)?;

        if total_amount_available < target_amount {
            return Ok(None);
        }

        let mut stmt = db_conn.prepare(
            "SELECT y, amount FROM proof WHERE node_id = ?1 AND state = ?2 ORDER BY amount DESC;",
        )?;
        let proofs_res_iterator = stmt.query_map(params![node_id, ProofState::Unspent], |r| {
            Ok((r.get::<_, PublicKey>(0)?, r.get::<_, Amount>(1)?))
        })?;

        for proof_res in proofs_res_iterator {
            let (y, proof_amount) = proof_res?;
            match remaining_amount.cmp(&proof_amount) {
                std::cmp::Ordering::Less => proofs_not_used.push((y, proof_amount)),
                std::cmp::Ordering::Equal => {
                    proofs_ids.push(y);
                    remaining_amount -= proof_amount;
                    break;
                }
                std::cmp::Ordering::Greater => {
                    proofs_ids.push(y);
                    remaining_amount -= proof_amount;
                }
            }
        }
    }

    if !remaining_amount.is_zero() {
        let proof_to_swap = proofs_not_used
            .iter()
            .rev()
            .find(|(_, a)| a > &remaining_amount)
            // We know that total_amount_available was >= target_amount
            // We know it cannot be equal to remaining amount otherwise we would have subtracted it
            // So there must be one greater stored in proofs_not_used
            .unwrap();

        let new_tokens = swap_to_have_target_amount(
            pool.clone(),
            node_client,
            node_id,
            unit,
            remaining_amount,
            proof_to_swap,
        )
        .await?;

        for token in new_tokens.into_iter().rev() {
            let token_amount = token.1;
            match remaining_amount.cmp(&token_amount) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Greater => {
                    proofs_ids.push(token.0);
                    remaining_amount -= token_amount;
                }
                std::cmp::Ordering::Equal => {
                    proofs_ids.push(token.0);
                    break;
                }
            }
        }
    }

    Ok(Some(proofs_ids))
}

pub fn load_tokens_from_db(
    db_conn: &Connection,
    proofs_ids: &[PublicKey],
) -> Result<nut00::Proofs, Error> {
    if proofs_ids.is_empty() {
        return Ok(vec![]);
    }

    let proofs = db::proof::get_proofs_by_ids(db_conn, proofs_ids)?
        .into_iter()
        .map(
            |(amount, keyset_id, unblinded_signature, secret)| -> Result<nut00::Proof, Error> {
                Ok(nut00::Proof {
                    amount,
                    keyset_id,
                    secret,
                    c: unblinded_signature,
                })
            },
        )
        .collect::<Result<Vec<_>, Error>>()?;

    db::proof::set_proofs_to_state(db_conn, proofs_ids, ProofState::Reserved)?;

    Ok(proofs)
}

pub async fn swap_to_have_target_amount(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    unit: &str,
    target_amount: Amount,
    proof_to_swap: &(PublicKey, Amount),
) -> Result<Vec<(PublicKey, Amount)>, Error> {
    let (keyset_id, input_unblind_signature) = {
        let db_conn = pool.get()?;
        let keyset_id = get_active_keyset_for_unit(&db_conn, node_id, unit)?;

        let input_unblind_signature =
            db::proof::get_proof_and_set_state_pending(&db_conn, proof_to_swap.0)?
                .ok_or(Error::ProofNotAvailable)?;
        (keyset_id, input_unblind_signature)
    };

    let pre_mints =
        PreMint::generate_for_amount(proof_to_swap.1, &SplitTarget::Value(target_amount))?;

    let inputs = vec![node_client::Proof {
        amount: proof_to_swap.1.into(),
        keyset_id: input_unblind_signature.0.to_bytes().to_vec(),
        secret: input_unblind_signature.2.to_string(),
        unblind_signature: input_unblind_signature.1.to_bytes().to_vec(),
    }];

    let outputs = build_outputs_from_premints(keyset_id.to_bytes(), &pre_mints);

    let swap_request = node_client::SwapRequest { inputs, outputs };
    let swap_request_hash = hash_swap_request(&swap_request);
    let swap_result = node_client.swap(swap_request).await;

    let db_conn = pool.get()?;
    let swap_response = match swap_result {
        Ok(r) => {
            db::proof::set_proof_to_state(&db_conn, proof_to_swap.0, ProofState::Spent)?;
            r.into_inner()
        }
        Err(e) => {
            // TODO: delete instead when invalid input
            db::proof::set_proof_to_state(&db_conn, proof_to_swap.0, ProofState::Unspent)?;
            return Err(e.into());
        }
    };

    let new_tokens = store_new_tokens(
        &db_conn,
        node_id,
        keyset_id,
        pre_mints.into_iter(),
        swap_response.signatures.into_iter(),
    )?;
    drop(db_conn);

    acknowledge(node_client, nuts::nut19::Route::Swap, swap_request_hash).await?;

    Ok(new_tokens)
}

pub async fn receive_wad(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    unit: &str,
    compact_keyset_proofs: Vec<CompactKeysetProofs>,
) -> Result<Amount, Error> {
    const INSERT_PROOF: &str = r#"
        INSERT INTO proof
            (y, node_id, keyset_id, amount, secret, unblind_signature, state)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT DO UPDATE
            SET state = excluded.state
    "#;
    let mut ys = Vec::with_capacity(compact_keyset_proofs.len());
    let mut total_amount = Amount::ZERO;
    let mut inputs = Vec::with_capacity(compact_keyset_proofs.len());
    let mut stmt_params = Vec::with_capacity(compact_keyset_proofs.len());

    for compact_keyset_proof in compact_keyset_proofs.into_iter() {
        let (keyset_unit, max_order) = read_or_import_node_keyset(
            pool.clone(),
            node_client,
            node_id,
            compact_keyset_proof.keyset_id,
        )
        .await?;
        if keyset_unit != unit {
            return Err(Error::UnitMissmatch(keyset_unit, unit.to_string()));
        }

        for compact_proof in compact_keyset_proof.proofs.into_iter() {
            let amount = u64::from(compact_proof.amount);
            if !amount.is_power_of_two() || amount == 0 {
                return Err(Error::Protocol(
                    "All proof amounts must be powers of two".to_string(),
                ));
            }
            if amount >= max_order {
                return Err(Error::Protocol(format!(
                    "Proof amount {} is not less than max_order {} for keyset {}",
                    amount, max_order, compact_keyset_proof.keyset_id
                )));
            }
            let y = hash_to_curve(compact_proof.secret.as_ref())?;
            ys.push(y);

            total_amount = total_amount
                .checked_add(&compact_proof.amount)
                .ok_or(Error::AmountOverflow)?;

            inputs.push(node_client::Proof {
                amount,
                keyset_id: compact_keyset_proof.keyset_id.to_bytes().to_vec(),
                secret: compact_proof.secret.to_string(),
                unblind_signature: compact_proof.c.to_bytes().to_vec(),
            });
            stmt_params.push((
                y,
                node_id,
                compact_keyset_proof.keyset_id,
                compact_proof.amount,
                compact_proof.secret,
                compact_proof.c,
                ProofState::Pending,
            ));
        }
    }
    let keyset_id = {
        let db_conn = pool.get()?;
        let mut insert_proof_stmt = db_conn.prepare(INSERT_PROOF)?;
        for params in stmt_params {
            insert_proof_stmt.execute(params)?;
        }
        get_active_keyset_for_unit(&db_conn, node_id, unit)?
    };

    let pre_mints = PreMint::generate_for_amount(total_amount, &SplitTarget::None)?;
    let outputs = build_outputs_from_premints(keyset_id.to_bytes(), &pre_mints);

    let swap_request = node_client::SwapRequest { inputs, outputs };
    let swap_request_hash = hash_swap_request(&swap_request);
    let swap_result = node_client.swap(swap_request).await;

    {
        let mut db_conn = pool.get()?;
        let swap_response = match swap_result {
            Ok(r) => r.into_inner(),
            Err(e) => {
                db::proof::delete_proofs(&db_conn, &ys)?;
                return Err(e.into());
            }
        };

        let tx = db_conn.transaction()?;
        db::proof::set_proofs_to_state(&tx, &ys, ProofState::Spent)?;
        let _new_tokens = store_new_tokens(
            &tx,
            node_id,
            keyset_id,
            pre_mints.into_iter(),
            swap_response.signatures.into_iter(),
        )?;
        tx.commit()?;
    }

    acknowledge(node_client, nuts::nut19::Route::Swap, swap_request_hash).await?;

    Ok(total_amount)
}

pub async fn register_node(
    pool: Pool<SqliteConnectionManager>,
    node_url: &NodeUrl,
) -> Result<(NodeClient<tonic::transport::Channel>, u32), Error> {
    let mut node_client = NodeClient::connect(node_url.to_string()).await?;

    let node_id = {
        let db_conn = pool.get()?;
        db::node::insert(&db_conn, node_url)?;
        db::node::get_id_by_url(&db_conn, node_url)?
    };

    refresh_node_keysets(pool, &mut node_client, node_id).await?;

    Ok((node_client, node_id))
}

#[cfg(feature = "tls")]
#[derive(thiserror::Error, Debug)]
pub enum TlsError {
    #[error("failed to build tls connector: {0}")]
    BuildConnector(openssl::error::ErrorStack),
    #[error("failed set ALPN protocols: {0}")]
    SetAlpnProtos(openssl::error::ErrorStack),
    #[error("failed to get the node's socket address: {0}")]
    Socket(#[from] std::io::Error),
    #[error("invalid uri")]
    Uri,
    #[error("failed to connect to the node: {0}")]
    Connect(#[from] tonic_tls::Error),
}

pub async fn connect_to_node(node_url: &NodeUrl) -> Result<NodeClient<Channel>, Error> {
    #[cfg(not(feature = "tls"))]
    let node_client = NodeClient::connect(node_url.0.to_string()).await?;

    #[cfg(feature = "tls")]
    let node_client = {
        let mut connector = openssl::ssl::SslConnector::builder(openssl::ssl::SslMethod::tls())
            .map_err(|e| Error::Tls(TlsError::BuildConnector(e)))?;
        // ignore server cert validation errors.
        connector.set_verify_callback(openssl::ssl::SslVerifyMode::PEER, |ok, ctx| {
            if !ok {
                let e = ctx.error();
                #[cfg(feature = "tls-allow-self-signed")]
                if e.as_raw() == openssl_sys::X509_V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT {
                    return true;
                }
                log::error!("verify failed with code {}: {}", e.as_raw(), e);
                return false;
            }
            true
        });
        connector
            .set_alpn_protos(tonic_tls::openssl::ALPN_H2_WIRE)
            .map_err(|e| Error::Tls(TlsError::SetAlpnProtos(e)))?;
        let ssl_conn = connector.build();
        let socket_address = node_url
            .0
            .socket_addrs(|| None)
            .map_err(|e| Error::Tls(TlsError::Socket(e)))?[0];
        let uri: tonic::transport::Uri = socket_address
            .to_string()
            .parse()
            .map_err(|_| Error::Tls(TlsError::Uri))?;

        let connector = tonic_tls::openssl::TlsConnector::new(
            uri.clone(),
            ssl_conn,
            // Safe to unwrap because NodeUrl guarantee it has a domain
            node_url.0.domain().unwrap().to_string(),
        );
        let channel = tonic_tls::new_endpoint()
            .connect_with_connector(connector)
            .await
            .map_err(|e| Error::Tls(TlsError::Connect(tonic_tls::Error::from(e))))?;

        NodeClient::new(channel)
    };

    Ok(node_client)
}

pub async fn acknowledge(
    node_client: &mut NodeClient<Channel>,
    route: Route,
    message_hash: u64,
) -> Result<(), Error> {
    node_client
        .acknowledge(Request::new(AcknowledgeRequest {
            path: route.to_string(),
            request_hash: message_hash,
        }))
        .await?;

    Ok(())
}

pub fn create_wad_from_proofs<U: Unit>(
    node_url: NodeUrl,
    unit: U,
    memo: Option<String>,
    proofs: Vec<Proof>,
) -> CompactWad<U> {
    let compact_proofs = proofs
        .into_iter()
        .chunk_by(|p| p.keyset_id)
        .into_iter()
        .map(|(keyset_id, proofs)| CompactKeysetProofs {
            keyset_id,
            proofs: proofs
                .map(|p| CompactProof {
                    amount: p.amount,
                    secret: p.secret,
                    c: p.c,
                })
                .collect(),
        })
        .collect();
    CompactWad {
        node_url,
        unit,
        memo,
        proofs: compact_proofs,
    }
}
