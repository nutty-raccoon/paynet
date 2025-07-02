use anyhow::Result;
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
