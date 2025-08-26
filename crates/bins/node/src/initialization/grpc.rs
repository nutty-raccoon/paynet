#[cfg(feature = "keyset-rotation")]
use node::KeysetRotationServiceServer;
use std::{collections::HashSet, net::SocketAddr};
use tonic::transport::Server;
use tower::ServiceBuilder;
use tower_otel::trace;
use tracing::instrument;

use futures::TryFutureExt;
use node::NodeServer;
use nuts::QuoteTTLConfig;
use signer::SignerClient;
use sqlx::Postgres;
use starknet_types::Unit;
use tonic::{service::LayerExt, transport::Channel};

use crate::{grpc_service::GrpcState, liquidity_sources::LiquiditySources};

use super::{Error, env_variables::EnvVariables};

#[instrument]
pub async fn launch_tonic_server_task(
    pg_pool: sqlx::Pool<Postgres>,
    signer_client: SignerClient<trace::Grpc<Channel>>,
    liquidity_sources: LiquiditySources<Unit>,
    env_vars: EnvVariables,
) -> Result<(SocketAddr, impl Future<Output = Result<(), crate::Error>>), super::Error> {
    let nuts_settings = super::nuts_settings::nuts_settings();
    let supported_units: HashSet<_> = nuts_settings
        .nut04
        .methods
        .iter()
        .map(|m| m.unit)
        .chain(nuts_settings.nut05.methods.iter().map(|m| m.unit))
        .collect();

    let ttl = env_vars.quote_ttl.unwrap_or(3600);
    let grpc_state = GrpcState::new(
        pg_pool,
        signer_client,
        nuts_settings,
        QuoteTTLConfig {
            mint_ttl: ttl,
            melt_ttl: ttl,
        },
        liquidity_sources,
    );
    let address = format!("[::0]:{}", env_vars.grpc_port)
        .parse()
        .map_err(Error::InvalidGrpcAddress)?;

    // TODO: take into account past keyset rotations
    // init node shared
    grpc_state
        .init_first_keysets(supported_units.into_iter(), 0, 32)
        .await?;

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
        .named_layer(NodeServer::new(grpc_state.clone()));

    let tonic_future = {
        let tonic_server = build_server(
            #[cfg(feature = "tls")]
            &env_vars,
        )
        .map_err(super::Error::BuildServer)?;
        let mut tonic_server = tonic_server.layer(tower_otel::metrics::HttpLayer::server(&meter));

        let router = tonic_server
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
