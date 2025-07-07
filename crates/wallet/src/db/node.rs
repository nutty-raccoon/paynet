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

pub fn insert(conn: &Connection, node_url: &NodeUrl) -> Result<usize> {
    conn.execute(
        "INSERT INTO node (url) VALUES (?1) ON CONFLICT DO NOTHING;",
        [node_url],
    )
}

pub fn get_id_by_url(conn: &Connection, node_url: &NodeUrl) -> Result<u32> {
    let mut stmt = conn.prepare("SELECT id FROM node WHERE url = ?1;")?;
    let id = stmt.query_row(params![node_url], |r| r.get::<_, u32>(0))?;

    Ok(id)
}

pub fn get_url_by_id(conn: &Connection, node_id: u32) -> Result<NodeUrl> {
    let mut stmt = conn.prepare("SELECT url FROM node WHERE id = ?1;")?;
    let url = stmt.query_row(params![node_id], |r| r.get::<_, NodeUrl>(0))?;

    Ok(url)
}

pub fn fetch_all(conn: &Connection) -> Result<Vec<(u32, NodeUrl)>> {
    let mut stmt = conn.prepare("SELECT id, url FROM node;")?;

    let rows = stmt.query_map((), |r| Ok((r.get::<_, u32>(0)?, r.get::<_, NodeUrl>(1)?)))?;

    rows.collect::<Result<Vec<_>>>()
}
