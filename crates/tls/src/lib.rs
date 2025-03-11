use std::path::Path;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
// use crate::tls::errors::TlsChannelError;
pub mod errors;

pub async fn create_secure_channel(url: String) -> Result<Channel, Box<dyn std::error::Error>> {
    let pem = tokio::fs::read("path/to/ca.pem").await?;
    let ca_cert = Certificate::from_pem(pem);

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(ca_cert)
        .domain_name("your.domain.com");

    // let channel = Channel::from_shared(url)?
    //     .tls_config(tls_config)?
    //     .connect()
    //     .await?;

    let channel_builder = Channel::from_shared(url)?
    .tls_config(tls_config)
    // .map_err(|err| errors::TlsChannelError::TlsConfiguration(err))
    ?
    .connect()
    .await
    // .map_err(|err|errors::TlsChannelError::ChannelCreation(err))
    ?;

    Ok(channel_builder)
}
