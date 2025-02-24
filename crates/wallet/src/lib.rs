pub mod db;
mod outputs;
pub mod types;

use std::str::FromStr;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use node::{
    GetKeysetsRequest, MeltRequest, MeltResponse, MeltState, MintQuoteRequest, MintQuoteResponse,
    MintQuoteState, MintRequest, MintResponse, NodeClient, QuoteStateRequest, SwapRequest,
    SwapResponse,
};
use nuts::Amount;
use nuts::dhke::{hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut00::{BlindedMessage, Proof};
use nuts::nut01::{PublicKey, SecretKey};
use nuts::nut02::KeysetId;
use rusqlite::{Connection, params};
use tonic::transport::Channel;
use types::PreMint;

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
    outputs
        .iter()
        .map(|o| node::BlindedMessage {
            amount: o.amount.into(),
            keyset_id: o.keyset_id.to_bytes().to_vec(),
            blinded_secret: o.blinded_secret.to_bytes().to_vec(),
        })
        .collect()
}

pub async fn create_mint_quote(
    db_conn: &mut Connection,
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

pub async fn swap(
    node_client: &mut NodeClient<Channel>,
    inputs: &[Proof],
    outputs: &[BlindedMessage],
) -> Result<SwapResponse> {
    let req = SwapRequest {
        inputs: convert_inputs(inputs),
        outputs: convert_outputs(outputs),
    };

    let resp = node_client.swap(req).await?;

    Ok(resp.into_inner())
}

pub async fn refresh_node_keysets(
    db_conn: &mut Connection,
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
        let mut x = node_client.clone();
        futures.push(async move {
            x.keys(node::GetKeysRequest {
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

pub fn get_active_keyset_for_unit(
    db_conn: &mut Connection,
    node_id: u32,
    unit: &str,
) -> Result<KeysetId> {
    let keyset_id_as_i64 =
        db::fetch_one_active_keyset_id_for_node_and_unit(db_conn, node_id, unit)?
            .ok_or(anyhow!("not matching keyset"))?;

    let keyset_id = KeysetId::from_bytes(&keyset_id_as_i64.to_be_bytes())?;

    Ok(keyset_id)
}

pub async fn mint_and_store_new_tokens(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
    node_id: u32,
    unit: &str,
    total_amount: u64,
) -> Result<()> {
    let keyset_id = get_active_keyset_for_unit(db_conn, node_id, unit)?;

    let pre_mints = PreMint::generate_for_amount(total_amount.into())?;

    let keyset_id_as_vec = keyset_id.to_bytes().to_vec();

    let outputs = pre_mints
        .iter()
        .map(|pm| node::BlindedMessage {
            amount: pm.amount.into(),
            keyset_id: keyset_id_as_vec.clone(),
            blinded_secret: pm.blinded_secret.to_bytes().to_vec(),
        })
        .collect();

    let mint_response = node_client
        .mint(node::MintRequest {
            method,
            quote: quote_id,
            outputs,
        })
        .await?
        .into_inner();

    let signatures_iterator = mint_response
        .signatures
        .into_iter()
        .map(|bs| PublicKey::from_slice(&bs.blind_signature))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let signature_iterator = pre_mints
        .into_iter()
        .zip(signatures_iterator)
        .map(|(pm, signature)| (signature, pm.secret, pm.r, pm.amount));

    // TODO: make the whole flow only be iterator, without collect
    construct_proofs(db_conn, node_id, keyset_id.as_i64(), signature_iterator)
}

pub fn construct_proofs(
    db_conn: &mut Connection,
    node_id: u32,
    keyset_id: i64,
    iterator: impl IntoIterator<Item = (PublicKey, Secret, SecretKey, Amount)>,
) -> Result<()> {
    let tx = db_conn.transaction()?;
    const GET_PUBKEY: &str = r#"
        SELECT pubkey FROM key WHERE keyset_id = ?1 and amount = ?2 LIMIT 1;
    "#;
    const INSERT_PROOF: &str = r#"
        INSERT INTO proof
            (y, node_id, keyset_id, amount, secret, unblind_signature, state)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, 1)
    "#;
    let mut get_pubkey_stmt = tx.prepare(GET_PUBKEY)?;
    let mut insert_proof_stmt = tx.prepare(INSERT_PROOF)?;

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
        ])?;
    }

    // We need to drop before we commit
    drop(insert_proof_stmt);
    drop(get_pubkey_stmt);
    tx.commit()?;

    Ok(())
}
