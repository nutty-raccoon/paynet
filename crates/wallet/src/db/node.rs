use rusqlite::{Connection, Result};

use crate::types::NodeUrl;
use rusqlite::params;

pub const CREATE_TABLE_NODE: &str = r#"
        CREATE TABLE IF NOT EXISTS node (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE
        );

        CREATE INDEX node_url ON node(url); 
    "#;

pub fn insert(conn: &Connection, node_url: NodeUrl) -> Result<u32> {
    conn.execute(
        "INSERT INTO node (url) VALUES (?1) ON CONFLICT DO NOTHING;",
        [&node_url],
    )?;

    let mut stmt = conn.prepare("SELECT id FROM node WHERE url = ?1;")?;
    let id = stmt.query_row(params![node_url], |r| r.get::<_, u32>(0))?;

    Ok(id)
}

pub fn fetch_all(conn: &Connection) -> Result<Vec<(u32, NodeUrl)>> {
    let mut stmt = conn.prepare("SELECT id, url FROM node;")?;

    let rows = stmt.query_map((), |r| Ok((r.get::<_, u32>(0)?, r.get::<_, NodeUrl>(1)?)))?;

    rows.collect::<Result<Vec<_>>>()
}
