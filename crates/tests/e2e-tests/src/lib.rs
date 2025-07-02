use std::path::PathBuf;

use anyhow::{Result, anyhow};
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

pub fn db_connection() -> Result<(r2d2::Pool<SqliteConnectionManager>, PathBuf)> {
    let db_path = if let Ok(env_path) = std::env::var("WALLET_DB_PATH") {
        PathBuf::from(env_path)
    } else {
        dirs::data_dir()
            .map(|mut dp| {
                dp.push("test-wallet.sqlite3");
                dp
            })
            .ok_or(anyhow!("couldn't find `data_dir` on this computer"))?
    };
    println!(
        "Using database at {:?}\n",
        db_path
            .as_path()
            .to_str()
            .ok_or(anyhow!("invalid db path"))?
    );

    let mut db_conn = rusqlite::Connection::open(db_path.clone())?;

    wallet::db::create_tables(&mut db_conn)?;
    let manager = SqliteConnectionManager::file(db_path.clone());
    let pool = r2d2::Pool::new(manager)?;

    Ok((pool, db_path))
}
