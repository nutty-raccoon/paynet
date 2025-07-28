use std::str::FromStr;

use bitcoin::bip32::Xpriv;
use rusqlite::{Connection, Result, params};

pub const CREATE_TABLE_WALLET: &str = r#"
    CREATE TABLE IF NOT EXISTS wallet (
        seed_phrase TEXT NOT NULL,
        private_key TEXT NOT NULL,
        created_at INTEGER,
        updated_at INTEGER,
        is_restored BOOLEAN NOT NULL
    );"#;

pub struct Wallet {
    pub seed_phrase: String,
    pub private_key: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_restored: bool,
}

pub fn create(conn: &Connection, wallet: Wallet) -> Result<()> {
    let sql = r#"
        INSERT INTO wallet (seed_phrase, private_key, created_at, updated_at, is_restored)
        VALUES (?, ?, ?, ?, ?)
    "#;

    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![
        wallet.seed_phrase,
        wallet.private_key,
        wallet.created_at,
        wallet.updated_at,
        wallet.is_restored
    ])?;

    Ok(())
}

pub fn get(conn: &Connection) -> Result<Option<Wallet>> {
    let sql = r#"
        SELECT seed_phrase, private_key, created_at, updated_at, is_restored
        FROM wallet
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallet = stmt.query_row(params![], |row| {
        Ok(Wallet {
            seed_phrase: row.get(0)?,
            private_key: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            is_restored: row.get(4)?,
        })
    })?;
    Ok(Some(wallet))
}

pub fn get_private_key(conn: &Connection) -> Result<Option<Xpriv>> {
    let sql = r#"
        SELECT private_key
        FROM wallet
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let pk: String = stmt.query_row(params![], |row| row.get(0))?;

    Ok(Some(Xpriv::from_str(&pk).unwrap()))
}

pub fn get_wallets(conn: &Connection) -> Result<Vec<Wallet>> {
    let sql = r#"
        SELECT seed_phrase, private_key, created_at, updated_at, is_restored
        FROM wallet
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallets = stmt
        .query_map(params![], |row| {
            Ok(Wallet {
                seed_phrase: row.get(0)?,
                private_key: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                is_restored: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<Wallet>>>()?;
    Ok(wallets)
}

pub fn count_wallets(conn: &Connection) -> Result<u32> {
    let sql = r#"
        SELECT COUNT(*) FROM wallet
    "#;
    let mut stmt = conn.prepare(sql)?;
    let count: u32 = stmt.query_row(params![], |row| row.get(0))?;
    Ok(count)
}
