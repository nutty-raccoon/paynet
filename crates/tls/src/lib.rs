use tonic::transport::{Channel, ClientTlsConfig, Certificate, Identity};
use tokio::fs;
use anyhow::Result;

pub async fn create_secure_channel(
    url: String,
    server_cert_path: &str,
    client_cert_path: Option<&str>,
    client_key_path: Option<&str>,
    domain_name: &str,
) -> Result<Channel> {
    let server_cert = fs::read(server_cert_path).await?;
    let ca_certificate = Certificate::from_pem(server_cert);

    let mut tls_config = ClientTlsConfig::new()
        .ca_certificate(ca_certificate)
        .domain_name(domain_name);

    if let (Some(cert_path), Some(key_path)) = (client_cert_path, client_key_path) {
        let client_cert = fs::read(cert_path).await?;
        let client_key = fs::read(key_path).await?;
        let client_identity = Identity::from_pem(client_cert, client_key);
        tls_config = tls_config.identity(client_identity);
    }

    let channel = Channel::from_shared(url)?
        .tls_config(tls_config)?
        .connect()
        .await?;

    Ok(channel)
}