use anyhow::Result;
use r2d2_sqlite::SqliteConnectionManager;
use test_utils::common::utils::EnvVariables;

pub fn read_env_variables() -> Result<EnvVariables> {
    let node_url = std::env::var("NODE_URL")?;
    let rpc_url = std::env::var("RPC_URL")?;
    let private_key = std::env::var("PRIVATE_KEY")?;
    let account_address = std::env::var("ACCOUNT_ADDRESS")?;

    Ok(EnvVariables {
        node_url,
        rpc_url,
        private_key,
        account_address,
    })
}

pub fn db_connection() -> Result<r2d2::Pool<SqliteConnectionManager>> {
    let manager = SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager)?;
    let mut db_conn = pool.get()?;
    wallet::db::create_tables(&mut db_conn)?;

    Ok(pool)
}
