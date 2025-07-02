use std::str::FromStr;

use anyhow::Result;
use concurrency_tests::read_env_variables;
use test_utils::concurrency::starknet::operations::{mint_same_output, mint_same_quote};
use wallet::{connect_to_node, types::NodeUrl};

#[tokio::test]
pub async fn same_output() -> Result<()> {
    let env = read_env_variables()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let node_client = connect_to_node(&node_url).await?;

    mint_same_output(node_client, env).await?;
    Ok(())
}

#[tokio::test]
pub async fn same_quote() -> Result<()> {
    let env = read_env_variables()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let node_client = connect_to_node(&node_url).await?;

    mint_same_quote(node_client, env).await?;
    Ok(())
}
