use anyhow::{Result, anyhow};
use node::KeysetRotationServiceClient;
use std::env;
use std::time::{Duration, Instant};
use tonic_health::pb::health_client::HealthClient;

use node::node_client::NodeClient;
use tonic::transport::Channel;

fn ensure_env_variables() -> Result<()> {
    if env::var("GRPC_PORT").is_ok()
        && env::var("NODE_GRPC_PORT").is_ok()
        && env::var("ROOT_KEY").is_ok()
    {
        return Ok(());
    }

    dotenvy::from_filename("signer.env")
        .map(|_| ())
        .map_err(|e| {
            anyhow!(
                "Environment variables not set and failed to load signer.env: {}",
                e
            )
        })?;

    dotenvy::from_filename("node.env").map(|_| ()).map_err(|e| {
        anyhow!(
            "Environment variables not set and failed to load node.env: {}",
            e
        )
    })
}

async fn connect_with_timeout(port_env_var: &str, label: &str) -> Result<Channel> {
    ensure_env_variables()?;
    let port = env::var(port_env_var)?;
    let address = format!("https://localhost:{}", port);
    let timeout = Instant::now() + Duration::from_secs(3);

    loop {
        match Channel::builder(address.parse()?).connect().await {
            Ok(channel) => return Ok(channel),
            Err(_) if Instant::now() > timeout => {
                return Err(anyhow!("timeout waiting for {}", label));
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

pub async fn get_signer_channel() -> Result<Channel> {
    connect_with_timeout("GRPC_PORT", "signer").await
}

pub async fn get_node_channel() -> Result<Channel> {
    connect_with_timeout("NODE_GRPC_PORT", "node").await
}

pub async fn init_health_client() -> Result<HealthClient<tonic::transport::Channel>> {
    let channel = get_signer_channel().await?;
    let client = tonic_health::pb::health_client::HealthClient::new(channel);

    Ok(client)
}

pub async fn init_signer_client() -> Result<signer::SignerClient<tonic::transport::Channel>> {
    let channel = get_signer_channel().await?;
    let client = signer::SignerClient::new(channel);

    Ok(client)
}

pub async fn init_node_client() -> Result<NodeClient<tonic::transport::Channel>> {
    let channel = get_node_channel().await?;
    let client = NodeClient::new(channel);

    Ok(client)
}

pub async fn init_keyset_client() -> Result<KeysetRotationServiceClient<tonic::transport::Channel>>
{
    let channel = get_node_channel().await?;
    let client = KeysetRotationServiceClient::new(channel);

    Ok(client)
}
