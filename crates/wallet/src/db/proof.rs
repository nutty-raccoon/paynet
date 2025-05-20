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

pub fn compute_total_amount_of_available_proofs(conn: &Connection, node_id: u32) -> Result<Amount> {
    let mut stmt = conn.prepare(
        r#"SELECT COALESCE(
                (SELECT SUM(amount) FROM proof WHERE node_id=?1 AND state=?2),
                0
              );"#,
    )?;
    let sum = stmt.query_row(params![node_id, ProofState::Unspent], |r| {
        r.get::<_, Amount>(0)
    })?;

    Ok(sum)
}

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
    let _ = conn.execute("UPDATE proof SET state = ?2 WHERE y = ?1", (y, state));

    Ok(())
}

pub fn set_proofs_to_state(conn: &Connection, ys: &[PublicKey], state: ProofState) -> Result<()> {
    // Build placeholder string like "?,?,?" based on number of items
    let mut placeholders = "?,".repeat(ys.len() - 1);
    placeholders.push('?');

    // Prepare the statement with dynamic placeholders
    let sql = format!("UPDATE proof SET state = ?1 WHERE y IN ({})", placeholders);
    let mut stmt = conn.prepare(&sql)?;

    // Bind state as first parameter
    stmt.raw_bind_parameter(1, state)?;
    // Bind each public key string to its respective placeholder
    for (i, y) in ys.iter().enumerate() {
        stmt.raw_bind_parameter(i + 2, y)?;
    }

    stmt.raw_execute()?;
    Ok(())
}

/// Return the proofs data related to the ids
///
/// Will error if any of those ids doesn't exist
/// The order of the returned proofs is not guaranteed to match the input `proof_ids`.
#[allow(clippy::type_complexity)]
pub fn get_proofs_by_ids(
    conn: &Connection,
    proof_ids: &[PublicKey],
) -> Result<Vec<(Amount, KeysetId, PublicKey, Secret)>> {
    if proof_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Dynamically create the placeholders (?, ?, ...)
    let mut placeholders = "?,".repeat(proof_ids.len() - 1);
    placeholders.push('?');
    let sql = format!(
        "SELECT amount, keyset_id, unblind_signature, secret FROM proof WHERE y IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&sql)?;

    // Create a slice of references to ToSql-compatible types
    for (i, y) in proof_ids.iter().enumerate() {
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

/// Returns the maximum allowed amount (max_order) for a given keyset_id from the key table.
pub fn get_max_order_for_keyset(
    conn: &rusqlite::Connection,
    keyset_id: nuts::nut02::KeysetId,
) -> rusqlite::Result<u64> {
    let mut stmt = conn.prepare("SELECT MAX(amount) FROM key WHERE keyset_id = ?1")?;
    let max_order: u64 = stmt
        .query_row([keyset_id], |row| row.get::<_, Option<u64>>(0))?
        .unwrap_or(0);
    Ok(max_order)
}
