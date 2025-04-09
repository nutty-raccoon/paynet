use rusqlite::{Connection, OptionalExtension, Result, ToSql, params};

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
    let placeholders = ys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("UPDATE proof SET state = ? WHERE y = in ({})", placeholders);
    let mut stmt = conn.prepare(&sql)?;
    let mut params = Vec::with_capacity(ys.len() + 1);
    params.push(&state as &dyn ToSql);
    params.extend(ys.iter().map(|pk| pk as &dyn ToSql));
    stmt.execute(&params[..])?;

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
    let placeholders = proof_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT amount, keyset_id, unblind_signature, secret FROM proof WHERE y IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&sql)?;

    // Create a slice of references to ToSql-compatible types
    let params_slice: Vec<&dyn ToSql> = proof_ids.iter().map(|pk| pk as &dyn ToSql).collect();

    let proofs = stmt
        .query_map(&params_slice[..], |r| {
            Ok((
                r.get::<_, Amount>(0)?,
                r.get::<_, KeysetId>(1)?,
                r.get::<_, PublicKey>(2)?,
                r.get::<_, Secret>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(proofs)
}
