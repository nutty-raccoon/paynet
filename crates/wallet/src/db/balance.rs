use crate::types::ProofState;
use rusqlite::{Connection, Result};

pub const GET_BALANCE_FOR_NODE: &str = r#"
    SELECT CAST(k.unit as TEXT), SUM(p.amount) as total_amount
    FROM proof p
    JOIN keyset k ON p.keyset_id = k.id
    WHERE p.node_id = ? AND p.state = ?
    GROUP BY k.unit"#;

pub fn get_balance_for_node(conn: &Connection, node_id: u32) -> Result<Vec<(String, u64)>> {
    let mut stmt = conn.prepare(GET_BALANCE_FOR_NODE)?;

    stmt.query_map([node_id, ProofState::Unspent as u32], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<Result<Vec<(String, u64)>>>()
}
