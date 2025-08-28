use rusqlite::{Connection, Result, params};

pub const CREATE_TABLE_WALLET: &str = r#"
    CREATE TABLE IF NOT EXISTS wallet (
        created_at INTEGER,
        updated_at INTEGER,
        is_restored BOOLEAN NOT NULL
    );"#;

pub struct Wallet {
    pub created_at: u64,
    pub updated_at: u64,
    pub is_restored: bool,
}

pub fn create(conn: &Connection, wallet: Wallet) -> Result<()> {
    let sql = r#"
        INSERT INTO wallet (created_at, updated_at, is_restored)
        VALUES (?, ?, ?)
    "#;

    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![
        wallet.created_at,
        wallet.updated_at,
        wallet.is_restored
    ])?;

    Ok(())
}

pub fn get(conn: &Connection) -> Result<Option<Wallet>> {
    let sql = r#"
        SELECT created_at, updated_at, is_restored
        FROM wallet
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallet = stmt.query_row(params![], |row| {
        Ok(Wallet {
            created_at: row.get(0)?,
            updated_at: row.get(1)?,
            is_restored: row.get(2)?,
        })
    })?;
    Ok(Some(wallet))
}

pub fn get_wallets(conn: &Connection) -> Result<Vec<Wallet>> {
    let sql = r#"
        SELECT created_at, updated_at, is_restored
        FROM wallet
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallets = stmt
        .query_map(params![], |row| {
            Ok(Wallet {
                created_at: row.get(0)?,
                updated_at: row.get(1)?,
                is_restored: row.get(2)?,
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
