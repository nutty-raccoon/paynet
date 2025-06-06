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

async fn get_signer_channel() -> Result<Channel> {
    ensure_env_variables()?;
    let signer_port = std::env::var("GRPC_PORT")?;

    let address = format!("https://localhost:{}", signer_port);

    let timeout = Instant::now() + Duration::from_secs(3);
    let channel = loop {
        if let Ok(c) = tonic::transport::Channel::builder(address.parse()?)
            .connect()
            .await
        {
            break c;
        }
        if Instant::now() > timeout {
            return Err(anyhow!("timeout waiting for signer"));
        }
    };

    Ok(channel)
}

async fn get_grpc_channel() -> Result<Channel> {
    ensure_env_variables()?;
    let signer_port = std::env::var("NODE_GRPC_PORT")?;

    let address = format!("https://localhost:{}", signer_port);

    let timeout = Instant::now() + Duration::from_secs(3);
    let channel = loop {
        if let Ok(c) = tonic::transport::Channel::builder(address.parse()?)
            .connect()
            .await
        {
            break c;
        }
        if Instant::now() > timeout {
            return Err(anyhow!("timeout waiting for signer"));
        }
    };

    Ok(channel)
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
    let channel = get_grpc_channel().await?;
    let client = NodeClient::new(channel);

    Ok(client)
}

pub async fn init_keyset_client() -> Result<KeysetRotationServiceClient<tonic::transport::Channel>>
{
    let channel = get_grpc_channel().await?;
    let client = KeysetRotationServiceClient::new(channel);

    Ok(client)
}
