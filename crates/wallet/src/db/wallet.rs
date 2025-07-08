use rusqlite::{Connection, Result, params};
pub struct Wallet {
    pub id: String,
    pub node_id: u32,
    pub counter: u64,
    pub keyset_id: String,
    pub seed_phrase: String,
    pub private_key: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_user_saved_locally: bool,
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
