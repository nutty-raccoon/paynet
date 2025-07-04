use std::str::FromStr;

use anyhow::Result;
use concurrency_tests::read_env_variables;
use test_utils::concurrency::starknet::operations::melt_same_input;
use wallet::{connect_to_node, types::NodeUrl};

#[tokio::test]
pub async fn same_intput() -> Result<()> {
    let env = read_env_variables()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let node_client = connect_to_node(&node_url).await?;

    melt_same_input(node_client, env).await?;
    Ok(())
}
