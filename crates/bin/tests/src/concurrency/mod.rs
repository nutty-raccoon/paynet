use std::str::FromStr;

use wallet::{connect_to_node, types::NodeUrl};

use crate::{
    concurrency::concurrence_ops::{blind_message_concurrence, swap_concurrence},
    env_variables::EnvVariables,
    errors::{Error, Result},
};

mod concurrence_ops;

pub async fn run_concurrency(env: EnvVariables) -> Result<()> {
    println!("[CONCURRENCY] Launching concurrency tests");
    let node_url = NodeUrl::from_str(&env.node_url).map_err(|e| Error::Other(e.into()))?;
    let node_client = connect_to_node(&node_url).await?;

    blind_message_concurrence(node_client.clone(), env.clone()).await?;
    swap_concurrence(node_client, env).await?;

    println!("✅ [CONCURRENCY] All tasks completed successfully — concurrency test passed.\n");
    Ok(())
}
