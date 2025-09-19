use cashu_client::CashuClient;
use num_traits::Zero;
use nuts::{Amount, nut01::PublicKey, traits::Unit};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use tracing::error;

use crate::{
    ConnectToNodeError, db, fetch_inputs_ids_from_db_or_node,
    types::{NodeUrl, ProofState, compact_wad::CompactWads},
    unprotected_load_tokens_from_db, wad,
    wallet::SeedPhraseManager,
};

#[derive(Debug, thiserror::Error)]
pub enum PlanSpendingError {
    #[error("failed to iteract with the database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("not enough funds available for unit {0}, requested: {1}, available: {2}")]
    NotEnoughFunds(String, Amount, Amount),
    #[error("duplicate node id {0} in prefered nodes ids")]
    DuplicatePreferedNodeId(u32),
}

pub fn plan_spending<U: Unit>(
    db_conn: &Connection,
    amount_to_send: Amount,
    unit: U,
    prefered_node_ids: &[u32],
) -> Result<Vec<(u32, Amount)>, PlanSpendingError> {
    // Check all prefered nodes are unique
    // Otherwise we will try to spend the same proofs twice :(
    for i in 0..prefered_node_ids.len() {
        if prefered_node_ids[i + 1..].contains(&prefered_node_ids[i]) {
            return Err(PlanSpendingError::DuplicatePreferedNodeId(
                prefered_node_ids[i],
            ));
        }
    }
    let mut amount_left_to_send = amount_to_send;

    let mut amount_per_node_id = Vec::new();
    for node_id in prefered_node_ids {
        let total_amount_available =
            db::proof::get_node_total_available_amount_of_unit(db_conn, *node_id, unit.as_ref())?;
        if total_amount_available < amount_left_to_send {
            amount_left_to_send -= total_amount_available;
            amount_per_node_id.push((*node_id, total_amount_available));
        } else {
            amount_per_node_id.push((*node_id, amount_left_to_send));
            amount_left_to_send = Amount::ZERO;
            break;
        }
    }

    if amount_left_to_send.is_zero() {
        return Ok(amount_per_node_id);
    }

    let ordered_nodes_and_amount = db::proof::get_nodes_ids_and_available_funds_ordered_desc(
        db_conn,
        unit.as_ref(),
        prefered_node_ids,
    )?;

    for (node_id, total_amount_available) in ordered_nodes_and_amount {
        if total_amount_available < amount_left_to_send {
            amount_left_to_send -= total_amount_available;
            amount_per_node_id.push((node_id, total_amount_available));
        } else {
            amount_per_node_id.push((node_id, amount_left_to_send));
            amount_left_to_send = Amount::ZERO;
            break;
        }
    }

    if !amount_left_to_send.is_zero() {
        return Err(PlanSpendingError::NotEnoughFunds(
            unit.to_string(),
            amount_to_send,
            amount_to_send - amount_left_to_send,
        ));
    }

    Ok(amount_per_node_id)
}

#[derive(Debug, thiserror::Error)]
pub enum GatherProofIdsFromNodeError {
    #[error("failed to get a connection from the pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("failed to connect to node: {0}")]
    ConnectToNode(#[from] ConnectToNodeError),
    #[error("failed to iteract with the database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("not enough funds for node {0}")]
    NotEnoughFunds(u32),
    #[error("failed to fetch inputs from db or node: {0}")]
    FetchInputs(crate::Error),
}

pub async fn gather_proofs_ids_for_node<U: Unit>(
    pool: Pool<SqliteConnectionManager>,
    seed_phrase_manager: impl SeedPhraseManager,
    node_client: &mut impl CashuClient,
    node_id: u32,
    amount: Amount,
    unit: U,
) -> Result<Vec<PublicKey>, GatherProofIdsFromNodeError> {
    let proofs_ids = fetch_inputs_ids_from_db_or_node(
        seed_phrase_manager,
        pool.clone(),
        node_client,
        node_id,
        amount,
        unit.as_ref(),
    )
    .await
    .map_err(GatherProofIdsFromNodeError::FetchInputs)?
    .ok_or(GatherProofIdsFromNodeError::NotEnoughFunds(node_id))?;

    Ok(proofs_ids)
}

#[derive(Debug, thiserror::Error)]
pub enum LoadProofsAndCreateWadsError {
    #[error("failed to load proofs form the database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
}

pub fn load_proofs_and_create_wads(
    db_conn: &Connection,
    nodes_with_proofs: Vec<(NodeUrl, Vec<PublicKey>)>,
    unit: &str,
    memo: Option<String>,
) -> Result<CompactWads, LoadProofsAndCreateWadsError> {
    let mut wads = Vec::with_capacity(nodes_with_proofs.len());
    let mut should_revert = None;

    for (i, (node_url, proofs_ids)) in nodes_with_proofs.iter().enumerate() {
        let proofs = match unprotected_load_tokens_from_db(db_conn, proofs_ids) {
            Ok(p) => p,
            Err(e) => {
                should_revert = Some((i, e));
                break;
            }
        };

        let wad = wad::create_from_parts(node_url.clone(), unit.to_string(), memo.clone(), proofs);
        if let Err(e) = db::wad::register_wad(
            db_conn,
            db::wad::WadType::OUT,
            &wad.node_url,
            &wad.memo,
            proofs_ids,
        ) {
            should_revert = Some((i, e));
            break;
        }
        wads.push(wad);
    }
    if let Some((max_reached, cause_error)) = should_revert {
        nodes_with_proofs
            .iter()
            .take(max_reached)
            .for_each(|(node_url, proofs_id)| {
                if let Err(e) =
                    db::proof::set_proofs_to_state(db_conn, proofs_id, ProofState::Unspent)
                {
                    error!(
                        "failed to revet state of the following proofs: {}\nProofs ids: {:?}",
                        e, proofs_id
                    );
                }
                if let Err(e) = db::wad::delete_wad(db_conn, node_url, proofs_id) {
                    error!(
                        "failed to revet state of wad: {}\nProofs ids: {:?}",
                        e, proofs_id
                    );
                }
            });

        Err(LoadProofsAndCreateWadsError::Rusqlite(cause_error))
    } else {
        Ok(CompactWads::new(wads))
    }
}
