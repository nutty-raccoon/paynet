use std::str::FromStr;

use anyhow::Result;
use e2e_tests::{db_connection, read_env_variables};
use test_utils::e2e::starknet::wallet_ops::WalletOps;
use wallet::types::NodeUrl;

#[tokio::test]
pub async fn run_e2e() -> Result<()> {
    let env = read_env_variables()?;
    let db_pool = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;

    let (node_client, node_id) = wallet::register_node(db_pool.clone(), &node_url).await?;

    let mut wallet_ops = WalletOps::new(db_pool.clone(), node_id, node_client);

    wallet_ops
        .mint(10.into(), starknet_types::Asset::Strk, env)
        .await?;
    let wad = wallet_ops
        .send(
            node_url,
            10.into(),
            starknet_types::Asset::Strk,
            Some("Here is some money".to_string()),
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

    Ok(())
}
