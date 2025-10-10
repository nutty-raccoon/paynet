#[cfg(feature = "keyset-rotation")]
use node::KeysetRotationServiceServer;
use std::net::SocketAddr;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tracing::instrument;

use futures::TryFutureExt;
use node::NodeServer;
use tonic::service::LayerExt;

use crate::{app_state::AppState, grpc_service::GrpcState};

use super::{Error, env_variables::EnvVariables};

#[instrument]
pub async fn launch_tonic_server_task(
    app_state: AppState,
    env_vars: u16,
) -> Result<(SocketAddr, impl Future<Output = Result<(), crate::Error>>), super::Error> {
    let address = format!("[::0]:{}", env_vars)
        .parse()
        .map_err(Error::InvalidGrpcAddress)?;

    // init health reporter service
    let health_service = {
        let (health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter.set_serving::<NodeServer<GrpcState>>().await;
        #[cfg(feature = "keyset-rotation")]
        health_reporter
            .set_serving::<KeysetRotationServiceServer<GrpcState>>()
            .await;

        health_service
    };
    let optl_layer = tower_otel::trace::GrpcLayer::server(tracing::Level::INFO);
    let meter = opentelemetry::global::meter(env!("CARGO_PKG_NAME"));

    #[cfg(feature = "keyset-rotation")]
    let keyset_rotation_service = ServiceBuilder::new()
        .layer(optl_layer.clone())
        .named_layer(KeysetRotationServiceServer::new(grpc_state.clone()));

    let node_service = ServiceBuilder::new()
        .layer(optl_layer)
        .named_layer(NodeServer::new(app_state.clone()));

    const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("node_service_desciptor");

    let reflexion_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let tonic_future = {
        let tonic_server = build_server(
            #[cfg(feature = "tls")]
            &env_vars,
        )
        .map_err(super::Error::BuildServer)?;
        let mut tonic_server = tonic_server.layer(tower_otel::metrics::HttpLayer::server(&meter));

        let router = tonic_server
            .add_service(reflexion_service)
            .add_service(health_service)
            .add_service(node_service);
        #[cfg(feature = "keyset-rotation")]
        let router = router.add_service(keyset_rotation_service);

        router.serve(address).map_err(crate::Error::Tonic)
    };

    Ok((address, tonic_future))
}

#[cfg(not(feature = "tls"))]
pub fn build_server() -> Result<Server, anyhow::Error> {
    tracing::info!("🚀 Starting gRPC server...");

    Ok(tonic::transport::Server::builder())
}

#[cfg(feature = "tls")]
pub fn build_server(env_vars: &EnvVariables) -> Result<Server, anyhow::Error> {
    let key_path = &env_vars.tls_key_path;
    let cert_path = &env_vars.tls_cert_path;
    // Load TLS certificates
    let cert = match std::fs::read(cert_path) {
        Ok(cert) => {
            tracing::info!("✅ TLS certificate loaded successfully from {}", cert_path);
            cert
        }
        Err(e) => {
            eprintln!("❌ Failed to load TLS certificate:");
            eprintln!("   Certificate: {}", cert_path);
            eprintln!("   Error: {}", e);
            eprintln!();
            eprintln!("🚫 gRPC server cannot start without valid HTTPS certificates");

            #[cfg(debug_assertions)]
            {
                eprintln!();
                eprintln!("💡 To generate local certificates with mkcert:");
                eprintln!("   1. Install mkcert: https://github.com/FiloSottile/mkcert");
                eprintln!("   2. Run: mkcert -install");
                eprintln!("   3. Run: mkdir -p certs");
                eprintln!(
                    "   4. Run: mkcert -key-file certs/key.pem -cert-file certs/cert.pem localhost 127.0.0.1 ::1"
                );
                eprintln!();
            }
            return Err(anyhow::anyhow!("Failed to load TLS certificate: {}", e));
        }
    };

    let key = match std::fs::read(key_path) {
        Ok(key) => {
            tracing::info!("✅ TLS private key loaded successfully from {}", key_path);
            key
        }
        Err(e) => {
            eprintln!("❌ Failed to load TLS private key:");
            eprintln!("   Private key: {}", key_path);
            eprintln!("   Error: {}", e);
            return Err(anyhow::anyhow!("Failed to load TLS private key: {}", e));
        }
    };

    let identity = tonic::transport::Identity::from_pem(cert, key);
    let tls_config = tonic::transport::ServerTlsConfig::new().identity(identity);

    tracing::info!("🔒 Starting gRPC server with TLS...");
    tracing::info!("📜 Certificate: {}", cert_path);
    tracing::info!("🔑 Private key: {}", key_path);

    let server = tonic::transport::Server::builder().tls_config(tls_config)?;

    Ok(server)
}
