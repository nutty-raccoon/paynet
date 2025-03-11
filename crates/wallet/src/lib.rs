pub mod db;
mod outputs;
pub mod types;

use std::collections::{HashMap, hash_map};
use std::str::FromStr;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use node::{
    GetKeysetsRequest, MintQuoteRequest, MintQuoteResponse, MintQuoteState, MintRequest,
    MintResponse, NodeClient, QuoteStateRequest,
};
use num_traits::CheckedAdd;
use nuts::dhke::{hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut00::{self, BlindedMessage, Proof};
use nuts::nut01::{PublicKey, SecretKey};
use nuts::nut02::KeysetId;
use nuts::{Amount, SplitTarget};
use rusqlite::{Connection, params};
use tonic::transport::Channel;

pub use db::create_tables;

pub fn convert_inputs(inputs: &[Proof]) -> Vec<node::Proof> {
    inputs
        .iter()
        .map(|p| node::Proof {
            amount: p.amount.into(),
            keyset_id: p.keyset_id.to_bytes().to_vec(),
            secret: p.secret.to_string(),
            unblind_signature: p.c.to_bytes().to_vec(),
        })
        .collect()
}

pub fn convert_outputs(outputs: &[BlindedMessage]) -> Vec<node::BlindedMessage> {
pub fn convert_outputs(outputs: &[BlindedMessage]) -> Vec<node::BlindedMessage> {
    outputs
        .iter()
        .map(|o| node::BlindedMessage {
            amount: o.amount.into(),
            keyset_id: o.keyset_id.to_bytes().to_vec(),
            blinded_secret: o.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub fn build_outputs_from_premints(
    keyset_id: [u8; 8],
    pre_mints: &[PreMint],
) -> Vec<node::BlindedMessage> {
    pre_mints
        .iter()
        .map(|pm| node::BlindedMessage {
            amount: pm.amount.into(),
            keyset_id: keyset_id.to_vec(),
            blinded_secret: pm.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub async fn create_mint_quote(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    amount: u64,
    unit: String,
) -> Result<MintQuoteResponse> {
    let response = node_client
        .mint_quote(MintQuoteRequest {
            method: method.clone(),
            amount,
            unit: unit.clone(),
            description: None,
        })
        .await?
        .into_inner();

    db::store_mint_quote(db_conn, method, amount, unit, &response)?;

    Ok(response)
}

pub async fn get_mint_quote_state(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<MintQuoteState> {
    let response = node_client
        .mint_quote_state(QuoteStateRequest {
            method,
            quote: quote_id,
        })
        .await?
        .into_inner();

    db::set_mint_quote_state(db_conn, response.quote, response.state)?;

    Ok(MintQuoteState::try_from(response.state)?)
}

pub async fn mint(
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote: String,
    outputs: &[BlindedMessage],
) -> Result<MintResponse> {
    let req = MintRequest {
        method,
        quote,
        outputs: convert_outputs(outputs),
    };

    let resp = node_client.mint(req).await?;

    Ok(resp.into_inner())
}

pub async fn refresh_node_keysets(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
) -> Result<()> {
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;

    let new_keyset_ids = crate::db::upsert_node_keysets(db_conn, node_id, keysets)?;

    // Parallelization of the queries
    let mut futures = futures::stream::FuturesUnordered::new();
    for new_keyset_id in new_keyset_ids {
        let mut cloned_node_client = node_client.clone();
        futures.push(async move {
            cloned_node_client
                .keys(node::GetKeysRequest {
                    keyset_id: Some(new_keyset_id.to_vec()),
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
                db::insert_keyset_keys(
                    db_conn,
                    keyset[0].id.clone().try_into().unwrap(),
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
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    keyset_id: KeysetId,
) -> Result<String> {
    let keyset_id_as_bytes = keyset_id.to_bytes();

    // Happy path, it is in DB
    if let Some(unit) = db::get_keyset_unit(db_conn, keyset_id_as_bytes)? {
        return Ok(unit);
    }

    let resp = node_client
        .keys(node::GetKeysRequest {
            keyset_id: Some(keyset_id_as_bytes.to_vec()),
        })
        .await?
        .into_inner();
    let keyset = resp.keysets.first().unwrap();

    let _ = db_conn.execute(
        "INSERT INTO keyset (id, node_id, unit, active) VALUES (?1, ?2, ?3, ?4)",
        (keyset_id_as_bytes, node_id, &keyset.unit, keyset.active),
    )?;

    db::insert_keyset_keys(
        db_conn,
        keyset_id_as_bytes,
        keyset.keys.iter().map(|k| (k.amount, k.pubkey.as_str())),
    )?;

    Ok(keyset.unit.clone())
}

pub fn get_active_keyset_for_unit(
    db_conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<KeysetId> {
    let keyset_id = db::fetch_one_active_keyset_id_for_node_and_unit(db_conn, node_id, unit)?
        .ok_or(anyhow!("not matching keyset"))?;

    let keyset_id = KeysetId::from_bytes(&keyset_id)?;

    Ok(keyset_id)
}

pub async fn mint_and_store_new_tokens(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
    node_id: u32,
    unit: &str,
    total_amount: u64,
) -> Result<()> {
    let keyset_id = get_active_keyset_for_unit(db_conn, node_id, unit)?;

    let pre_mints = PreMint::generate_for_amount(total_amount.into(), &SplitTarget::None)?;

    let outputs = build_outputs_from_premints(keyset_id.to_bytes(), &pre_mints);

    let mint_response = node_client
        .mint(node::MintRequest {
            method,
            quote: quote_id,
            outputs,
        })
        .await?
        .into_inner();

    let _ = store_new_tokens(
        db_conn,
        node_id,
        keyset_id,
        pre_mints.into_iter(),
        mint_response.signatures.into_iter(),
    );

    Ok(())
}

pub fn store_new_tokens(
    db_conn: &Connection,
    node_id: u32,
    keyset_id: KeysetId,
    pre_mints: impl Iterator<Item = PreMint>,
    signatures: impl Iterator<Item = node::BlindSignature>,
) -> Result<Vec<(PublicKey, Amount)>> {
    let signatures_iterator = signatures
        .into_iter()
        .map(|bs| PublicKey::from_slice(&bs.blind_signature))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let signature_iterator = pre_mints
        .into_iter()
        .zip(signatures_iterator)
        .map(|(pm, signature)| (signature, pm.secret, pm.r, pm.amount));

    // TODO: make the whole flow only be iterator, without collect
    construct_proofs_from_blind_signatures(
        db_conn,
        node_id,
        keyset_id.to_bytes(),
        signature_iterator,
    )
}

pub fn construct_proofs_from_blind_signatures(
    db_conn: &Connection,
    node_id: u32,
    keyset_id: [u8; 8],
    iterator: impl IntoIterator<Item = (PublicKey, Secret, SecretKey, Amount)>,
) -> Result<Vec<(PublicKey, Amount)>> {
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

    for (blinded_message, secret, r, amount) in iterator {
        let pubkey = get_pubkey_stmt.query_row(params![keyset_id, u64::from(amount)], |row| {
            row.get::<_, String>(0)
        })?;
        let mint_pubkey = PublicKey::from_str(&pubkey)?;
        let unblinded_signature: PublicKey = unblind_message(&blinded_message, &r, &mint_pubkey)?;

        let y = hash_to_curve(secret.as_ref())?;

        insert_proof_stmt.execute(params![
            &y.to_bytes(),
            node_id,
            keyset_id,
            u64::from(amount),
            secret.to_string(),
            &unblinded_signature.to_bytes(),
            ProofState::Unspent,
        ])?;

        new_tokens.push((y, amount));
    }

    Ok(new_tokens)
}

pub async fn fetch_inputs_from_db_or_node(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    target_amount: u64,
    unit: &str,
) -> Result<Option<Vec<nut00::Proof>>> {
    let total_amount_available =
        db::proof::compute_total_amount_of_available_proofs(db_conn, node_id)?;

    if total_amount_available < target_amount {
        return Ok(None);
    }

    let mut stmt = db_conn.prepare(
        "SELECT y, amount FROM proof WHERE node_id = ?1 AND state = ?2 ORDER BY amount DESC;",
    )?;
    let proofs_res_iterator = stmt.query_map(params![node_id, ProofState::Unspent], |r| {
        Ok((r.get::<_, [u8; 33]>(0)?, r.get::<_, u64>(1)?))
    })?;

    let mut proofs_ids = Vec::new();
    let mut proofs_not_used = Vec::new();
    let mut remaining_amount = target_amount;
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
    drop(stmt);

    if remaining_amount != 0 {
        let proof_to_swap = proofs_not_used
            .iter()
            .rev()
            .find(|(_, a)| a > &remaining_amount)
            // We know that total_amount_available was target_amount
            // We know it cannot be equal to remaining amout otherwas we would have substracted it
            // So there must be one greater stored in proof_not_used
            .unwrap();

        let (new_tokens_keyset_id, pre_mints, swap_response) = swap_to_have_target_amount(
            db_conn,
            node_client,
            node_id,
            unit,
            remaining_amount,
            proof_to_swap,
        )
        .await?;

        let new_tokens = store_new_tokens(
            db_conn,
            node_id,
            new_tokens_keyset_id,
            pre_mints.into_iter(),
            swap_response.signatures.into_iter(),
        )?;

        for token in new_tokens.into_iter().rev() {
            let token_amount = u64::from(token.1);
            match remaining_amount.cmp(&token_amount) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Greater => {
                    proofs_ids.push(token.0.to_bytes());
                    remaining_amount -= token_amount;
                }
                std::cmp::Ordering::Equal => {
                    proofs_ids.push(token.0.to_bytes());
                    break;
                }
            }
        }
    }

    let proofs = db::proof::get_proofs_by_ids(db_conn, &proofs_ids)?
        .into_iter()
        .map(
            |(amount, keyset_id, unblinded_signature, secret)| -> Result<nut00::Proof> {
                Ok(nut00::Proof {
                    amount: amount.into(),
                    keyset_id: KeysetId::from_bytes(&keyset_id)?,
                    secret: Secret::new(secret),
                    c: PublicKey::from_slice(&unblinded_signature)?,
                })
            },
        )
        .collect::<Result<Vec<_>>>()?;

    db::proof::set_proofs_to_state(db_conn, proofs_ids.iter(), ProofState::Reserved)?;

    Ok(Some(proofs))
}

pub async fn swap_to_have_target_amount(
    db_conn: &Connection,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    unit: &str,
    target_amount: u64,
    proof_to_swap: &([u8; 33], u64),
) -> Result<(KeysetId, Vec<PreMint>, node::SwapResponse)> {
    let keyset_id = get_active_keyset_for_unit(db_conn, node_id, unit)?;

    let input_unblind_signature =
        db::proof::get_proof_and_set_state_pending(db_conn, proof_to_swap.0)?
            .ok_or(anyhow!("proof not available anymore"))?;

    let pre_mints = PreMint::generate_for_amount(
        Amount::from(proof_to_swap.1),
        &SplitTarget::Value(target_amount.into()),
    )?;

    let inputs = vec![node::Proof {
        amount: proof_to_swap.1,
        keyset_id: input_unblind_signature.0.to_vec(),
        secret: input_unblind_signature.2.clone(),
        unblind_signature: input_unblind_signature.1.to_vec(),
    }];

    let outputs = build_outputs_from_premints(keyset_id.to_bytes(), &pre_mints);

    let swap_response = match node_client
        .swap(node::SwapRequest { inputs, outputs })
        .await
    {
        Ok(r) => {
            db::proof::set_proof_to_state(db_conn, proof_to_swap.0, ProofState::Spent)?;
            r.into_inner()
        }
        Err(e) => {
            // TODO: delete instead ?
            db::proof::set_proof_to_state(db_conn, proof_to_swap.0, ProofState::Unspent)?;
            return Err(e.into());
        }
    };

    Ok((keyset_id, pre_mints, swap_response))
}

pub async fn receive_wad(
    db_conn: &Connection,
    unit: String,
    request: String,
    inputs: &[Proof],
) -> Result<MeltResponse> {
    let req = MeltRequest {
        method,
        unit,
        request,
        inputs: convert_inputs(inputs),
    };
    let resp = node_client.melt(req).await?;

    Ok(resp.into_inner())
}

pub async fn swap(
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    proofs: &[nut00::Proof],
) -> Result<HashMap<String, Amount>> {
    const INSERT_PROOF: &str = r#"
        INSERT INTO proof
            (y, node_id, keyset_id, amount, secret, unblind_signature, state)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT DO UPDATE
            SET state = excluded.state
    "#;
    let mut insert_proof_stmt = db_conn.prepare(INSERT_PROOF)?;
    let mut keyset_id_to_unit = HashMap::new();
    let mut unit_to_amount = HashMap::new();
    let mut ys = Vec::with_capacity(proofs.len());

    for proof in proofs.iter() {
        let y = hash_to_curve(proof.secret.as_ref())?;
        ys.push(y.to_bytes());

        insert_proof_stmt.execute(params![
            &y.to_bytes(),
            node_id,
            proof.keyset_id.to_bytes(),
            u64::from(proof.amount),
            proof.secret.to_string(),
            proof.c.to_bytes(),
            ProofState::Pending
        ])?;

        let unit = match keyset_id_to_unit.entry(proof.keyset_id) {
            hash_map::Entry::Occupied(occupied_entry) => {
                let unit: &String = occupied_entry.get();
                unit.clone()
            }
            hash_map::Entry::Vacant(vacant_entry) => {
                let unit =
                    read_or_import_node_keyset(db_conn, node_client, node_id, proof.keyset_id)
                        .await?;

                vacant_entry.insert(unit.clone());
                unit
            }
        };

        let entry = unit_to_amount.entry(unit).or_insert(Amount::ZERO);
        *entry = entry
            .checked_add(&proof.amount)
            .ok_or(anyhow!("amount overflow"))?;
    }

    let inputs = convert_inputs(proofs);

    let mut outputs = Vec::new();
    let mut unit_to_premints = HashMap::new();
    for (unit, amount) in unit_to_amount.iter() {
        let pre_mints = PreMint::generate_for_amount(*amount, &SplitTarget::None)?;
        let keyset_id = get_active_keyset_for_unit(db_conn, node_id, unit)?;
        outputs.extend_from_slice(
            build_outputs_from_premints(keyset_id.to_bytes(), &pre_mints).as_slice(),
        );
        unit_to_premints.insert(unit, (keyset_id, pre_mints));
    }

    let swap_response = match node_client
        .swap(node::SwapRequest { inputs, outputs })
        .await
    {
        Ok(r) => {
            db::proof::set_proofs_to_state(db_conn, ys.iter(), ProofState::Spent)?;
            r.into_inner()
        }
        Err(e) => {
            // TODO: delete instead?
            db::proof::set_proofs_to_state(db_conn, ys.iter(), ProofState::Unspent)?;
            return Err(e.into());
        }
    };

    for (_unit, (keyset_id, pre_mints)) in unit_to_premints.into_iter() {
        let _new_tokens = store_new_tokens(
            db_conn,
            node_id,
            keyset_id,
            pre_mints.into_iter(),
            swap_response.signatures.iter().filter_map(|s| {
                if s.keyset_id == keyset_id.to_bytes() {
                    Some(s.clone())
                } else {
                    None
                }
            }),
        )?;
    }

    Ok(unit_to_amount)
}

pub async fn register_node(
    db_conn: &Connection,
    node_url: NodeUrl,
) -> Result<(NodeClient<tonic::transport::Channel>, u32)> {
    let mut node_client = NodeClient::connect(&node_url).await?;

    let node_id = db::node::insert(db_conn, &node_url)?;
    refresh_node_keysets(db_conn, &mut node_client, node_id).await?;

    Ok((node_client, node_id))
}
