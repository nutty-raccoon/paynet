use std::env::VarError;

use super::Error;

pub fn read_env_variables() -> Result<EnvVariables, Error> {
    let node_url = std::env::var("NODE_URL").map_err(|_e| VarError::NotPresent)?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_e| VarError::NotPresent)?;
    let private_key = std::env::var("PRIVATE_KEY").map_err(|_e| VarError::NotPresent)?;
    let account_address = std::env::var("ACCOUNT_ADDRESS").map_err(|_e| VarError::NotPresent)?;

    Ok(EnvVariables {
        node_url,
        rpc_url,
        private_key,
        account_address,
    })
}

#[derive(Debug)]
pub struct EnvVariables {
    pub node_url: String,
    pub rpc_url: String,
    pub private_key: String,
    pub account_address: String,
}
