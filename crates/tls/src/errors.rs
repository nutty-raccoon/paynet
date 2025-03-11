// use tonic::transport::{Certificate, Channel, ClientTlsConfig};
// use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum TlsChannelError {
//     #[error("failed to read CA certificate: {0}")]
//     CertificateRead(#[from] std::io::Error),

//     #[error("failed to configure TLS: {0}")]
//     TlsConfiguration(#[source] tonic::transport::Error),

//     #[error("failed to create secure channel: {0}")]
//     ChannelCreation(#[source] tonic::transport::Error),
// }