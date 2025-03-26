use crate::types::{Amount, ProofState};
use rusqlite::{Connection, Result};

pub fn get_for_node(conn: &Connection, node_id: u32) -> Result<Vec<(String, Amount)>> {
    let mut stmt = conn.prepare(
        r#"SELECT CAST(k.unit as TEXT), SUM(p.amount) as total_amount
           FROM node n
           LEFT JOIN proof p ON p.node_id = n.id AND p.state = ?
           LEFT JOIN keyset k ON p.keyset_id = k.id
           WHERE n.id = ?
           AND p.node_id IS NOT NULL
           GROUP BY k.unit
           HAVING total_amount > 0"#,
    )?;

    stmt.query_map([ProofState::Unspent, node_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect()
}

pub fn get_for_all_nodes(conn: &Connection) -> Result<Vec<(i64, String, Vec<(String, Amount)>)>> {
    let sql = r#"
        SELECT n.id, n.url, k.unit, SUM(p.amount) as amount
        FROM node n
        LEFT JOIN proof p ON p.node_id = n.id AND p.state = ?
        LEFT JOIN keyset k ON p.keyset_id = k.id
        GROUP BY n.id, n.url, k.unit
        HAVING amount > 0
        ORDER BY n.id
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([ProofState::Unspent], |row| {
        Ok((
            row.get(0)?, // node_id
            row.get(1)?, // url
            row.get(2)?, // unit
            row.get(3)?, // amount
        ))
    })?;

    let mut result: Vec<(i64, String, Vec<(String, Amount)>)> = Vec::new();

    for row in rows {
        let (node_id, url, unit, amount) = row?;

        match result.last_mut() {
            Some((id, _, balances)) if &node_id == id => {
                balances.push((unit, amount));
            }
            Some(_) | None => result.push((node_id, url, vec![(unit, amount)])),
        }
    }

    Ok(result)
}
