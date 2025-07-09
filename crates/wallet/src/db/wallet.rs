use rusqlite::{Connection, Result, params};
pub struct Wallet {
    pub seed_phrase: String,
    pub private_key: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_user_seed_backed: bool,
}

pub fn create_wallet(conn: &Connection, wallet: Wallet) -> Result<()> {
    let sql = r#"
        INSERT INTO settings (seed_phrase, private_key, created_at, updated_at, is_user_seed_backed)
        VALUES (?, ?, ?, ?, ?)
    "#;

    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![wallet.seed_phrase, wallet.private_key, wallet.created_at, wallet.updated_at, wallet.is_user_seed_backed])?;

    Ok(())
}

pub fn get_wallet(conn: &Connection) -> Result<Option<Wallet>> {
    let sql = r#"
        SELECT seed_phrase, private_key, created_at, updated_at, is_user_seed_backed
        FROM settings
        ORDER BY RANDOM()
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallet = stmt.query_row(params![], |row| {
        Ok(Wallet {
            seed_phrase: row.get(0)?,
            private_key: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            is_user_seed_backed: row.get(4)?,
        })
    })?;
    Ok(Some(wallet))
}

pub fn get_wallet_by_seed_phrase(conn: &Connection, seed_phrase: String) -> Result<Option<Wallet>> {
    let sql = r#"
        SELECT seed_phrase, private_key, created_at, updated_at, is_user_seed_backed
        FROM settings
        WHERE seed_phrase = ?
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallet = stmt.query_row(params![seed_phrase], |row| {
        Ok(Wallet {
            seed_phrase: row.get(0)?,
            private_key: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            is_user_seed_backed: row.get(4)?,
        })
    })?;
    Ok(Some(wallet))
}

pub fn update_wallet(conn: &Connection, wallet: Wallet) -> Result<()> {
    let sql = r#"
        UPDATE settings
        SET private_key = ?, updated_at = ?, is_user_seed_backed = ?, seed_phrase = ?
    "#;
    let mut stmt = conn.prepare(sql)?;
    stmt.execute(params![wallet.private_key, wallet.updated_at, wallet.is_user_seed_backed, wallet.seed_phrase])?;
    Ok(())
}

pub fn get_wallets(conn: &Connection) -> Result<Vec<Wallet>> {
    let sql = r#"
        SELECT seed_phrase, private_key, created_at, updated_at, is_user_seed_backed
        FROM settings
        ORDER BY RANDOM()
        LIMIT 1
    "#;
    let mut stmt = conn.prepare(sql)?;
    let wallets = stmt.query_map(params![], |row| {
        Ok(Wallet {
            seed_phrase: row.get(0)?,
            private_key: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            is_user_seed_backed: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<Wallet>>>()?;
    Ok(wallets)
}


pub fn count_wallets(conn: &Connection) -> Result<u32> {
    let sql = r#"
        SELECT COUNT(*) FROM settings
    "#;
    let mut stmt = conn.prepare(sql)?;
    let count: u32 = stmt.query_row(params![], |row| row.get(0))?;
    Ok(count)
}
