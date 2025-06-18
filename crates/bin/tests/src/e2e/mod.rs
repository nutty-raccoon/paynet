use std::{path::PathBuf, str::FromStr};

use crate::errors::{Error, Result};
use anyhow::anyhow;
use rusqlite::Connection;
use wallet::types::NodeUrl;

use crate::{e2e::wallet_ops::WalletOps, env_variables::EnvVariables};

mod wallet_ops;

fn db_connection() -> Result<(Connection, PathBuf)> {
    let db_path = if let Ok(env_path) = std::env::var("WALLET_DB_PATH") {
        PathBuf::from(env_path)
    } else {
        dirs::data_dir()
            .map(|mut dp| {
                dp.push("cli-wallet.sqlite3");
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
    Ok((db_conn, db_path))
}

pub async fn run_e2e(env: EnvVariables) -> Result<()> {
    let (mut db_conn, _db_path) = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url).map_err(|e| Error::Other(e.into()))?;

    let tx = db_conn.transaction()?;

    let (node_client, node_id) = wallet::register_node(&tx, node_url.clone()).await?;
    tx.commit()?;

    let mut wallet_ops = WalletOps::new(db_conn, node_id, node_client);

    wallet_ops
        .mint(10.into(), starknet_types::Asset::Strk, env)
        .await?;
    let wad = wallet_ops
        .send(
            node_url,
            10.into(),
            starknet_types::Asset::Strk,
            Some("./test.wad".to_string()),
        )
        .await?;
    wallet_ops.receive(&wad).await?;
    wallet_ops
        .melt(
            10.into(),
            starknet_types::Asset::Strk,
            "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
        )
        .await?;

    println!("✅ [E2E] Test passed: funds successfully mint, sent, received and melt");
    Ok(())
}
