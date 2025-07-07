use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};
use tonic_health::pb::health_client::HealthClient;

use node_client::keyset_rotation_service_client::KeysetRotationServiceClient;
use node_client::node_client::NodeClient;

use tonic::transport::Channel;

async fn get_grpc_channel() -> Result<Channel> {
    let grpc_port = std::env::var("GRPC_PORT")?;
    let endpoint = format!("http://[::0]:{}", grpc_port);

    let timeout = Instant::now() + Duration::from_secs(10);

    let channel = loop {
        if let Ok(c) = tonic::transport::Channel::builder(endpoint.parse()?)
            .connect()
            .await
        {
            break c;
        }
        if Instant::now() > timeout {
            return Err(anyhow!("timeout waiting for node"));
        }
    };
    Ok(channel)
}

pub async fn init_health_client() -> Result<HealthClient<tonic::transport::Channel>> {
    let channel = get_grpc_channel().await?;
    let client = tonic_health::pb::health_client::HealthClient::new(channel);

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
