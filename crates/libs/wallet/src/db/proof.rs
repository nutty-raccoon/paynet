use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::types::ProofState;
use nuts::{Amount, nut00::secret::Secret, nut01::PublicKey, nut02::KeysetId};

pub const CREATE_TABLE_PROOF: &str = r#"
        CREATE TABLE IF NOT EXISTS proof (
            y BLOB(33) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            keyset_id BLOB(8) REFERENCES keyset(id) ON DELETE CASCADE,
            amount INTEGER NOT NULL,
            secret TEXT UNIQUE NOT NULL,
            unblind_signature BLOB(33) UNIQUE NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3, 4))
        );

        CREATE INDEX proof_node_id ON proof(node_id);
        CREATE INDEX proof_amount ON proof(amount);
        CREATE INDEX proof_state ON proof(state);
    "#;

/// Fetch the proof info and set it to pending
///
/// Will return None if the proof is already Pending.
#[allow(clippy::type_complexity)]
pub fn get_proof_and_set_state_pending(
    conn: &Connection,
    y: PublicKey,
) -> Result<Option<(KeysetId, PublicKey, Secret)>> {
    let n_rows = conn.execute(
        "UPDATE proof SET state = ?2 WHERE y = ?1 AND state == ?3 ;",
        (y, ProofState::Pending, ProofState::Unspent),
    )?;
    let values = if n_rows == 0 {
        None
    } else {
        let mut stmt =
            conn.prepare("SELECT keyset_id, unblind_signature , secret FROM proof WHERE y = ?1")?;

        stmt.query_row([y], |r| {
            Ok((
                r.get::<_, KeysetId>(0)?,
                r.get::<_, PublicKey>(1)?,
                r.get::<_, Secret>(2)?,
            ))
        })
        .optional()?
    };

    Ok(values)
}

pub fn set_proof_to_state(conn: &Connection, y: PublicKey, state: ProofState) -> Result<()> {
    conn.execute("UPDATE proof SET state = ?2 WHERE y = ?1", (y, state))?;

    Ok(())
}

fn build_ys_placeholder_string_for_in_statement(len: usize) -> String {
    // Build placeholder string like "?,?,?" based on number of items
    let mut placeholders = "?,".repeat(len - 1);
    placeholders.push('?');
    placeholders
}

pub fn set_proofs_to_state(
    conn: &Connection,
    ys: &[PublicKey],
    state: ProofState,
) -> Result<usize> {
    let placeholders = build_ys_placeholder_string_for_in_statement(ys.len());

    // Prepare the statement with dynamic placeholders
    let sql = format!("UPDATE proof SET state = ?1 WHERE y IN ({})", placeholders);
    let mut stmt = conn.prepare(&sql)?;

    // Bind state as first parameter
    stmt.raw_bind_parameter(1, state)?;
    // Bind each public key string to its respective placeholder
    for (i, y) in ys.iter().enumerate() {
        stmt.raw_bind_parameter(i + 2, y)?;
    }

    let rows_affected = stmt.raw_execute()?;
    Ok(rows_affected)
}

/// Return the proofs data related to the ids
///
/// Will error if any of those ids doesn't exist
/// The order of the returned proofs is not guaranteed to match the input `proof_ids`.
#[allow(clippy::type_complexity)]
pub fn get_proofs_by_ids(
    conn: &Connection,
    ys: &[PublicKey],
) -> Result<Vec<(Amount, KeysetId, PublicKey, Secret)>> {
    if ys.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = build_ys_placeholder_string_for_in_statement(ys.len());
    let sql = format!(
        "SELECT amount, keyset_id, unblind_signature, secret FROM proof WHERE y IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&sql)?;

    for (i, y) in ys.iter().enumerate() {
        stmt.raw_bind_parameter(i + 1, y)?;
    }

    let proofs = stmt
        .raw_query()
        .mapped(|r| -> Result<(Amount, KeysetId, PublicKey, Secret)> {
            {
                Ok((
                    r.get::<_, Amount>(0)?,
                    r.get::<_, KeysetId>(1)?,
                    r.get::<_, PublicKey>(2)?,
                    r.get::<_, Secret>(3)?,
                ))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(proofs)
}

/// Return the proofs state related to the ids
///
/// Will error if any of those ids doesn't exist
/// The order of the returned proofs is not guaranteed to match the input `proof_ids`.
pub fn get_proofs_state_by_ids(conn: &Connection, ys: &[PublicKey]) -> Result<Vec<ProofState>> {
    if ys.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = build_ys_placeholder_string_for_in_statement(ys.len());
    let sql = format!("SELECT state FROM proof WHERE y IN ({})", placeholders);

    let mut stmt = conn.prepare(&sql)?;

    for (i, y) in ys.iter().enumerate() {
        stmt.raw_bind_parameter(i + 1, y)?;
    }

    let proofs = stmt
        .raw_query()
        .mapped(|r| -> Result<ProofState> { r.get::<_, ProofState>(0) })
        .collect::<Result<Vec<_>>>()?;

    Ok(proofs)
}

/// Returns the maximum allowed amount (max_order) for a given keyset_id from the key table.
pub fn get_max_order_for_keyset(
    conn: &rusqlite::Connection,
    keyset_id: nuts::nut02::KeysetId,
) -> rusqlite::Result<Option<u64>> {
    let mut stmt = conn.prepare("SELECT MAX(amount) FROM key WHERE keyset_id = ?1")?;
    let max_order = stmt.query_row([keyset_id], |row| row.get::<_, Option<u64>>(0))?;

    Ok(max_order)
}

pub fn delete_proofs(conn: &Connection, ys: &[PublicKey]) -> Result<()> {
    let placeholders = build_ys_placeholder_string_for_in_statement(ys.len());
    let sql = format!("DELETE FROM proof WHERE y IN ({})", placeholders);
    let mut stmt = conn.prepare(&sql)?;
    for (i, y) in ys.iter().enumerate() {
        stmt.raw_bind_parameter(i + 1, y)?;
    }

    stmt.raw_execute()?;

    Ok(())
}

/// Returns the node available amount of unit
///
/// Sum the amount of each unspent proof of unit for this node
pub fn get_node_total_available_amount_of_unit(
    conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<Amount> {
    let mut stmt = conn.prepare(
        r#"SELECT COALESCE(
                (SELECT SUM(p.amount)
                 FROM proof p
                 JOIN keyset k ON p.keyset_id = k.id
                 WHERE p.node_id = ?1 AND p.state = ?2 AND k.unit = ?3),
                0
              );"#,
    )?;
    let sum = stmt.query_row(params![node_id, ProofState::Unspent, unit], |r| {
        r.get::<_, Amount>(0)
    })?;

    Ok(sum)
}

/// Returns the non excluded nodes ids along with their available funds
///
/// Will return the list of all nodes present in the database,
/// that have not been excleded by the `nodes_to_exclude` argument,
/// sorted by descending amount of unit available
pub fn get_nodes_ids_and_available_funds_ordered_desc(
    conn: &Connection,
    unit: &str,
    nodes_to_exclude: &[u32],
) -> Result<Vec<(u32, Amount)>> {
    // Create placeholders for the excluded nodes
    let placeholders = if nodes_to_exclude.is_empty() {
        String::new()
    } else {
        format!(
            "AND p.node_id NOT IN ({})",
            vec!["?"; nodes_to_exclude.len()].join(",")
        )
    };

    let query = format!(
        r#"SELECT p.node_id, COALESCE(SUM(p.amount), 0) as total_amount
           FROM proof p
           JOIN keyset k ON p.keyset_id = k.id
           WHERE p.state = ?1 AND k.unit = ?2 {}
           GROUP BY p.node_id
           ORDER BY total_amount DESC"#,
        placeholders
    );

    let mut stmt = conn.prepare(&query)?;
    stmt.raw_bind_parameter(1, ProofState::Unspent)?;
    stmt.raw_bind_parameter(2, unit)?;
    for (i, node_id) in nodes_to_exclude.iter().enumerate() {
        stmt.raw_bind_parameter(i + 3, node_id)?;
    }

    let res = stmt
        .raw_query()
        .mapped(|row| Ok((row.get::<_, u32>(0)?, row.get::<_, Amount>(1)?)))
        .collect::<Result<Vec<_>>>()?;

    Ok(res)
}
