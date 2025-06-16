use crate::types::ProofState;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

pub fn get_for_node(conn: &Connection, node_id: u32) -> Result<Vec<Balance>> {
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

    stmt.query_map(params![ProofState::Unspent, node_id], |row| {
        Ok(Balance {
            unit: row.get(0)?,
            amount: row.get(1)?,
        })
    })?
    .collect()
}

#[derive(Serialize, Deserialize)]
pub struct NodeData {
    pub id: i64,
    pub url: String,
    pub balances: Vec<Balance>,
}

#[derive(Serialize, Deserialize)]
pub struct Balance {
    pub unit: String,
    pub amount: i64,
}

pub fn get_for_all_nodes(conn: &Connection) -> Result<Vec<NodeData>> {
    let sql = r#"
        SELECT n.id, n.url, k.unit, SUM(p.amount) as amount
        FROM node n
        LEFT JOIN proof p ON p.node_id = n.id AND p.state = ?
        LEFT JOIN keyset k ON p.keyset_id = k.id
        GROUP BY n.id, n.url, k.unit
        ORDER BY n.id
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![ProofState::Unspent], |row| {
        Ok((
            row.get(0)?,                      // node_id
            row.get(1)?,                      // url
            row.get::<_, Option<String>>(2)?, // unit
            row.get::<_, Option<i64>>(3)?,    // amount
        ))
    })?;

    let mut result: Vec<NodeData> = Vec::new();

    for row in rows {
        let (node_id, url, opt_unit, opt_amount) = row?;

        match result.last_mut() {
            Some(NodeData {
                id,
                url: _,
                balances,
            }) if &node_id == id => {
                if let (Some(unit), Some(amount)) = (opt_unit, opt_amount) {
                    balances.push(Balance { unit, amount });
                }
            }
            Some(_) | None => {
                let mut node_balances = NodeData {
                    id: node_id,
                    url,
                    balances: vec![],
                };
                if let (Some(unit), Some(amount)) = (opt_unit, opt_amount) {
                    node_balances.balances.push(Balance { unit, amount });
                }

                result.push(node_balances);
            }
        }
    }

    Ok(result)
}
