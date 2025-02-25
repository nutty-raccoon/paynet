use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::types::ProofState;

pub const CREATE_TABLE_PROOF: &str = r#"
        CREATE TABLE IF NOT EXISTS proof (
            y BLOB(33) PRIMARY KEY,
            node_id TEXT NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            keyset_id BLOB(8) REFERENCES keyset(id) ON DELETE CASCADE,
            amount INTEGER NOT NULL,
            secret TEXT NOT NULL,
            unblind_signature BLOB(33) NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3, 4))
        );

        CREATE INDEX proof_node_id ON proof(node_id); 
        CREATE INDEX proof_amount ON proof(amount); 
        CREATE INDEX proof_state ON proof(state); 
    "#;

pub fn compute_total_amount_of_available_proofs(conn: &Connection, node_id: u32) -> Result<u64> {
    let mut stmt = conn.prepare(
        r#"SELECT COALESCE(
                (SELECT SUM(amount) FROM proof WHERE node_id=?1 AND state=?2),
                0
              );"#,
    )?;
    let sum = stmt.query_row(params![node_id, ProofState::Unspent], |r| {
        r.get::<_, u64>(0)
    })?;

    Ok(sum)
}

/// Fetch the proof info and set it to pending
///
/// Will return None if the proof is already Pending.
#[allow(clippy::type_complexity)]
pub fn get_proof_and_set_state_pending(
    conn: &Connection,
    y: [u8; 33],
) -> Result<Option<([u8; 8], [u8; 33], String)>> {
    let n_rows = conn.execute(
        "UPDATE proof SET state = ?2 WHERE y = ?1 AND state != ?2;",
        (y, ProofState::Pending),
    )?;
    let values = if n_rows == 0 {
        None
    } else {
        let mut stmt =
            conn.prepare("SELECT keyset_id, unblind_signature , secret FROM proof WHERE y = ?1")?;

        stmt.query_row([y], |r| {
            Ok((
                r.get::<_, [u8; 8]>(0)?,
                r.get::<_, [u8; 33]>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .optional()?
    };

    Ok(values)
}

pub fn set_proof_to_state(conn: &Connection, y: [u8; 33], state: ProofState) -> Result<()> {
    let _ = conn.execute("UPDATE proof SET state = ?2 WHERE y = ?1", (y, state));

    Ok(())
}
pub fn set_proofs_to_state(
    conn: &Connection,
    ys: impl Iterator<Item = [u8; 33]>,
    state: ProofState,
) -> Result<()> {
    let mut stmt = conn.prepare("UPDATE proof SET state = ?2 WHERE y = ?1")?;

    for y in ys {
        stmt.execute(params![y, state])?;
    }

    Ok(())
}

/// Return the proofs data related to the ids
///
/// Will error if any of those ids doesn't exist
#[allow(clippy::type_complexity)]
pub fn get_proofs_by_ids(
    conn: &Connection,
    proof_ids: &[[u8; 33]],
) -> Result<Vec<(u64, [u8; 8], [u8; 33], String)>> {
    let mut stmt = conn
        .prepare("SELECT amount, keyset_id, unblind_signature, secret FROM proof WHERE y = ?1")?;

    let mut proofs = Vec::with_capacity(proof_ids.len());
    for id in proof_ids {
        let proof = stmt.query_row([id], |r| {
            Ok((
                r.get::<_, u64>(0)?,
                r.get::<_, [u8; 8]>(1)?,
                r.get::<_, [u8; 33]>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        proofs.push(proof);
    }

    Ok(proofs)
}
