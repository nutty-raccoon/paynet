use crate::errors::Result;

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

#[derive(Debug, Clone)]
pub struct EnvVariables {
    pub node_url: String,
    pub rpc_url: String,
    pub private_key: String,
    pub account_address: String,
}
