use nuts::nut02::KeysetId;
use rusqlite::{Connection, Result, params};

pub mod balance;
pub mod keyset;
pub mod melt_quote;
pub mod mint_quote;
pub mod node;
pub mod proof;
pub mod wad;

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



pub const CREATE_TABLE_WAD: &str = r#"
        CREATE TABLE IF NOT EXISTS wad (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL CHECK (type IN ('IN', 'OUT')),
            status TEXT NOT NULL CHECK (status IN ('PENDING', 'CANCELLED', 'FINISHED', 'FAILED')),
            wad_data TEXT NOT NULL,
            total_amount_json TEXT NOT NULL,
            memo TEXT,
            created_at INTEGER NOT NULL,
            modified_at INTEGER NOT NULL
        );

        CREATE INDEX wad_type ON wad(type);
        CREATE INDEX wad_status ON wad(status);
        CREATE INDEX wad_created_at ON wad(created_at);
    "#;

pub const CREATE_TABLE_WAD_PROOF: &str = r#"
        CREATE TABLE IF NOT EXISTS wad_proof (
            wad_id INTEGER NOT NULL REFERENCES wad(id) ON DELETE CASCADE,
            proof_y BLOB(33) NOT NULL REFERENCES proof(y) ON DELETE CASCADE,
            PRIMARY KEY (wad_id, proof_y)
        );
    "#;

pub fn create_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute(node::CREATE_TABLE_NODE, ())?;
    tx.execute(keyset::CREATE_TABLE_KEYSET, ())?;
    tx.execute(CREATE_TABLE_KEY, ())?;
    tx.execute(CREATE_TABLE_MINT_QUOTE, ())?;
    tx.execute(CREATE_TABLE_MELT_QUOTE, ())?;
    tx.execute(proof::CREATE_TABLE_PROOF, ())?;
    tx.execute(CREATE_TABLE_WAD, ())?;
    tx.execute(CREATE_TABLE_WAD_PROOF, ())?;

    tx.commit()?;

    Ok(())
}

pub fn insert_keyset_keys<'a>(
    conn: &Connection,
    keyset_id: KeysetId,
    keys: impl Iterator<Item = (u64, &'a str)>,
) -> Result<()> {
    const INSERT_NEW_KEY: &str = r#"
        INSERT INTO key (keyset_id, amount, pubkey) VALUES (?1, ?2, ?3) ON CONFLICT DO NOTHING;
    "#;

    let mut stmt = conn.prepare(INSERT_NEW_KEY)?;
    for (amount, pk) in keys {
        stmt.execute(params![keyset_id, amount, pk])?;
    }

    Ok(())
}
