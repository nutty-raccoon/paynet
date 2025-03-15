use anyhow::{Result, anyhow};
use std::env;
use std::time::{Duration, Instant};
use tonic_health::pb::health_client::HealthClient;

use tonic::transport::Channel;

fn ensure_env_variables() -> Result<()> {
    if env::var("SOCKET_PORT").is_ok() && env::var("ROOT_KEY").is_ok() {
        return Ok(());
    }

    match dotenvy::from_filename("signer.env") {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(
            "Environment variables not set and failed to load signer.env: {}",
            e
        )),
    }
}

async fn get_signer_channel() -> Result<Channel> {
    ensure_env_variables()?;
    let signer_port = std::env::var("SOCKET_PORT")?;

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
