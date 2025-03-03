use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};
use tonic_health::pb::health_client::HealthClient;

use tonic::transport::Channel;

async fn get_signer_channel() -> Result<Channel> {
    let signer_port = std::env::var("SOCKET_PORT")?;

    let address = format!("http://localhost:{}", signer_port);

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
