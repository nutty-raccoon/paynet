use std::str::FromStr;

use wallet::{connect_to_node, types::NodeUrl};

use crate::{
    concurrency::concurrence_ops::{
        melt_same_input, mint_same_output, mint_same_quote, swap_same_input, swap_same_output,
    },
    env_variables::EnvVariables,
    errors::{Error, Result},
};

mod concurrence_ops;
mod utils;

pub async fn run_concurrency(env: EnvVariables) -> Result<()> {
    println!("[CONCURRENCY] Launching concurrency tests");
    let node_url = NodeUrl::from_str(&env.node_url).map_err(|e| Error::Other(e.into()))?;
    let node_client = connect_to_node(&node_url).await?;

    println!("\nrunning mint concurency test \n");
    melt_same_input(node_client.clone(), env.clone()).await?;
    println!("\nrunning mint concurency test \n");
    mint_same_output(node_client.clone(), env.clone()).await?;
    mint_same_quote(node_client.clone(), env.clone()).await?;
    println!("\nrunning swap concurency test\n");
    swap_same_input(node_client.clone(), env.clone()).await?;
    swap_same_output(node_client, env).await?;

    println!("✅ [CONCURRENCY] All tasks completed successfully — concurrency test passed.\n");
    Ok(())
}
