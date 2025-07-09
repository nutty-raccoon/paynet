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

pub fn get_wallet(conn: &Connection, seed_phrase: String) -> Result<Option<Wallet>> {
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
