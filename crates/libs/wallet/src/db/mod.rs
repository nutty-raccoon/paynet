use nuts::nut02::KeysetId;
use rusqlite::{Connection, Result, params};

pub mod balance;
pub mod keyset;
pub mod melt_quote;
pub mod mint_quote;
pub mod node;
pub mod proof;
pub mod wad;
pub mod wallet;

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
            expiry INTEGER NOT NULL,
            pubkey TEXT,
            secret_key TEXT
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

    tx.execute(wallet::CREATE_TABLE_WALLET, ())?;
    tx.execute(node::CREATE_TABLE_NODE, ())?;
    tx.execute(keyset::CREATE_TABLE_KEYSET, ())?;
    tx.execute(CREATE_TABLE_KEY, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(CREATE_TABLE_MELT_QUOTE, ())?;
    tx.execute(proof::CREATE_TABLE_PROOF, ())?;
    tx.execute(wad::CREATE_TABLE_WAD, ())?;
    tx.execute(wad::CREATE_TABLE_WAD_PROOF, ())?;

    tx.commit()?;

    Ok(())
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
