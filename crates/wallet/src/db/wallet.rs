use crate::types::{NodeUrl, ProofState};
use nuts::{Amount, traits::Unit};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAllNodesData {
    pub id: u32,
    pub url: NodeUrl,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub unit: String,
    pub amount: Amount,
}

pub fn get_for_all_nodes(conn: &Connection) -> Result<Vec<GetForAllNodesData>> {
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
            row.get::<_, Option<Amount>>(3)?, // amount
        ))
    })?;

    let mut result: Vec<GetForAllNodesData> = Vec::new();

    for row in rows {
        let (node_id, url, opt_unit, opt_amount) = row?;

        match result.last_mut() {
            Some(GetForAllNodesData {
                id,
                url: _,
                balances,
            }) if &node_id == id => {
                if let (Some(unit), Some(amount)) = (opt_unit, opt_amount) {
                    balances.push(Balance { unit, amount });
                }
            }
            Some(_) | None => {
                let mut node_balances = GetForAllNodesData {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAllNodesByUnitData {
    pub id: u32,
    pub url: NodeUrl,
    pub amount: Amount,
}

pub struct Wallet {
    pub id: String,
    pub node_id: u32,
    pub seed_phrase: String,
    pub private_key: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_user_saved_locally: bool,
    pub counter: u64,
}

pub fn create_wallet(conn: &Connection, wallet: Wallet) -> Result<()> {
    let sql = r#"
        INSERT INTO wallet (id, node_id, seed_phrase, private_key, created_at, updated_at, is_user_saved_locally, counter)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![wallet.id, wallet.node_id, wallet.seed_phrase, wallet.private_key, wallet.created_at, wallet.updated_at, wallet.is_user_saved_locally, wallet.counter])?;

    Ok(())
}

pub fn get_for_all_nodes_by_unit<U: Unit>(
    conn: &Connection,
    unit: U,
) -> Result<Vec<GetForAllNodesByUnitData>> {
    let sql = r#"
        SELECT n.id, n.url, SUM(p.amount) as amount
        FROM node n
        LEFT JOIN proof p ON p.node_id = n.id AND p.state = $1
        LEFT JOIN keyset k ON p.keyset_id = k.id
        WHERE unit = $2
        GROUP BY n.id, n.url, k.unit
        ORDER BY amount
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![ProofState::Unspent, unit.to_string()], |row| {
        Ok((
            row.get(0)?,                      // node_id
            row.get(1)?,                      // url
            row.get::<_, Option<Amount>>(2)?, // amount
        ))
    })?;

    let mut result: Vec<GetForAllNodesByUnitData> = Vec::new();

    for row in rows {
        let (node_id, url, opt_amount) = row?;
        if let Some(amount) = opt_amount {
            result.push(GetForAllNodesByUnitData {
                id: node_id,
                url,
                amount,
            });
        }
    }

    Ok(result)
}
