use nuts::nut02::KeysetId;
use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::types::NodeUrl;

pub mod balance;
pub mod melt_quote;
pub mod mint_quote;
pub mod node;
pub mod proof;

pub const CREATE_TABLE_KEYSET: &str = r#"
        CREATE TABLE IF NOT EXISTS keyset (
            id BLOB(8) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            unit TEXT NOT NULL,
            active BOOL NOT NULL
        );

        CREATE INDEX keyset_node_id ON keyset(node_id);
        CREATE INDEX keyset_unit ON keyset(unit);
        CREATE INDEX keyset_active ON keyset(active);
    "#;
pub const CREATE_TABLE_KEY: &str = r#"
        CREATE TABLE IF NOT EXISTS key (
            keyset_id BLOB(8) NOT NULL REFERENCES keyset(id) ON DELETE CASCADE,
            amount INTEGER NOT NULL,
            pubkey BLOB(33) NOT NULL,
            PRIMARY KEY (keyset_id, amount)
        );
    "#;
pub const CREATE_TABLE_MINT_QUOTE: &str = r#"
        CREATE TABLE IF NOT EXISTS mint_quote (
            id BLOB(16) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            method TEXT NOT NULL,
            amount INTEGER NOT NULL,
            unit TEXT NOT NULL,
            request TEXT NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
            expiry INTEGER NOT NULL
        );"#;
pub const CREATE_TABLE_MELT_QUOTE: &str = r#"
        CREATE TABLE IF NOT EXISTS melt_quote (
            id BLOB(16) PRIMARY KEY,
            node_id INTEGER NOT NULL REFERENCES node(id) ON DELETE CASCADE,
            method TEXT NOT NULL,
            amount INTEGER NOT NULL,
            unit TEXT NOT NULL,
            request TEXT NOT NULL,
            state INTEGER NOT NULL CHECK (state IN (1, 2, 3)),
            expiry INTEGER NOT NULL,
            transfer_ids TEXT
        );"#;

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute(node::CREATE_TABLE_NODE, ())?;
    tx.execute(CREATE_TABLE_KEYSET, ())?;
    tx.execute(CREATE_TABLE_KEY, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(CREATE_TABLE_MELT_QUOTE, ())?;
    tx.execute(proof::CREATE_TABLE_PROOF, ())?;

    tx.commit()?;

    Ok(())
}

pub fn upsert_node_keysets(
    conn: &Connection,
    node_id: u32,
    keysets: Vec<node_client::Keyset>,
) -> Result<Vec<KeysetId>> {
    conn.execute(
        r#"
        CREATE TEMPORARY TABLE IF NOT EXISTS _tmp_inserted (id INTEGER PRIMARY KEY);
        INSERT INTO _tmp_inserted (id) SELECT id FROM keyset;"#,
        (),
    )?;

    const UPSERT_NODE_KEYSET: &str = r#"
            INSERT INTO keyset (id, node_id, unit, active)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE
                SET active=excluded.active
                WHERE active != excluded.active;
    "#;

    for keyset in keysets {
        let id = KeysetId::from_bytes(&keyset.id).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                keyset.id.len(),
                rusqlite::types::Type::Blob,
                Box::new(e),
            )
        })?;
        conn.execute(
            UPSERT_NODE_KEYSET,
            params![id, node_id, keyset.unit, keyset.active],
        )?;
    }

    const GET_NEW_KEYSETS: &str = r#"
        SELECT id FROM keyset WHERE id NOT IN(SELECT id FROM _tmp_inserted);
    "#;

    let new_keyset_ids = {
        let mut stmt = conn.prepare(GET_NEW_KEYSETS)?;
        stmt.query_map([], |row| row.get::<_, KeysetId>(0))?
            .collect::<Result<Vec<_>>>()?
    };

    conn.execute("DELETE FROM _tmp_inserted", [])?;

    Ok(new_keyset_ids)
}

pub fn fetch_one_active_keyset_id_for_node_and_unit(
    conn: &Connection,
    node_id: u32,
    unit: &str,
) -> Result<Option<KeysetId>> {
    const FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT: &str = r#"
        SELECT id FROM keyset WHERE node_id = ? AND active = TRUE AND unit = ? LIMIT 1;
    "#;

    let mut stmt = conn.prepare(FETCH_ONE_ACTIVE_KEYSET_FOR_NODE_AND_UNIT)?;
    let result = stmt
        .query_row(params![node_id, unit], |row| row.get::<_, KeysetId>(0))
        .optional()?;

    Ok(result)
}

pub fn insert_keyset_keys<'a>(
    conn: &Connection,
    keyset_id: KeysetId,
    keys: impl Iterator<Item = (u64, &'a str)>,
) -> Result<()> {
    const INSET_NEW_KEY: &str = r#"
        INSERT INTO key (keyset_id, amount, pubkey) VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING;
    "#;

    let mut stmt = conn.prepare(INSET_NEW_KEY)?;
    for (amount, pk) in keys {
        stmt.execute(params![keyset_id, amount, pk])?;
    }

    Ok(())
}

pub fn get_node_url(conn: &Connection, node_id: u32) -> Result<Option<NodeUrl>> {
    let mut stmt = conn.prepare("SELECT url FROM node WHERE id = ?1 LIMIT 1")?;
    let opt_url = stmt
        .query_row([node_id], |r| r.get::<_, NodeUrl>(0))
        .optional()?;

    Ok(opt_url)
}

pub fn get_keyset_unit(conn: &Connection, keyset_id: KeysetId) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT unit FROM keyset WHERE id = ?1 LIMIT 1")?;
    let opt_unit = stmt
        .query_row(params![keyset_id], |r| r.get::<_, String>(0))
        .optional()?;

    Ok(opt_unit)
}
