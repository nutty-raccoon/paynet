use num_traits::Zero;
use nuts::{Amount, traits::Unit};
use rusqlite::Connection;

use crate::db;

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
