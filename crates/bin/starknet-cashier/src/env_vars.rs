use starknet::core::types::Felt;
use std::str::FromStr;
use url::Url;

// Environment variable names
pub const STARKNET_RPC_URL_ENV_VAR: &str = "STARKNET_RPC_URL";
pub const SOCKET_PORT_ENV_VAR: &str = "SOCKET_PORT";
pub const SIGNER_PRIVATE_KEY_ENV_VAR: &str = "SIGNER_PRIVATE_KEY";
pub const ACCOUNT_ADDRESS_ENV_VAR: &str = "ACCOUNT_ADDRESS";

// Function to read all required environment variables
pub fn read_env_variables() -> anyhow::Result<(Url, Felt, Felt, String)> {
    // Get RPC URL
    let rpc_url = Url::parse(
        &std::env::var(STARKNET_RPC_URL_ENV_VAR).expect("env var `STARKNET_RPC_URL` should be set"),
    )
    .expect("env var `STARKNET_RPC_URL` should be a valid url");

    // Get signer private key
    let private_key = Felt::from_str(
        &std::env::var(SIGNER_PRIVATE_KEY_ENV_VAR)
            .expect("env var `SIGNER_PRIVATE_KEY` should be set"),
    )?;

    // Get account address
    let address = Felt::from_str(
        &std::env::var(ACCOUNT_ADDRESS_ENV_VAR).expect("env var `ACCOUNT_ADDRESS` should be set"),
    )?;

    // Get socket port
    let socket_port =
        std::env::var(SOCKET_PORT_ENV_VAR).expect("env var `SOCKET_PORT` should be set");

    Ok((rpc_url, private_key, address, socket_port))
}
