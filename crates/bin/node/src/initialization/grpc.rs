#[cfg(feature = "keyset-rotation")]
use node::KeysetRotationServiceServer;
use std::net::SocketAddr;
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
        .init_first_keysets(&[Unit::MilliStrk], 0, 32)
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
        let mut tonic_server = tonic::transport::Server::builder()
            .layer(tower_otel::metrics::HttpLayer::server(&meter));

        let router = tonic_server
            .add_service(health_service)
            .add_service(node_service);
        #[cfg(feature = "keyset-rotation")]
        let router = router.add_service(keyset_rotation_service);

        // create future
        #[cfg(not(feature = "tls"))]
        let future = router.serve(address).map_err(crate::Error::Tonic);
        #[cfg(feature = "tls")]
        let future = router
            .serve_with_incoming(init_incoming(
                address,
                env_vars.tls_cert_path,
                env_vars.tls_key_path,
            )?)
            .map_err(crate::Error::Tonic);

        future
    };

    Ok((address, tonic_future))
}

#[cfg(feature = "tls")]
fn init_incoming(
    address: SocketAddr,
    tls_cert_path: String,
    tls_key_path: String,
) -> Result<tonic_tls::openssl::TlsIncoming, super::Error> {
    use openssl::pkey::PKey;
    use openssl::x509::X509;

    let cert = std::fs::read(tls_cert_path).expect("Failed to read tsl certificate");
    let key = std::fs::read(tls_key_path).expect("Failed to read tsl key");
    let cert = X509::from_pem(&cert)?;
    let key = PKey::private_key_from_pem(&key)?;

    let mut acceptor =
        openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())?;
    acceptor.set_private_key(&key)?;
    acceptor.set_certificate(&cert)?;
    acceptor.cert_store_mut().add_cert(cert.clone())?;
    acceptor.check_private_key()?;
    // Require HTTP/2
    acceptor.set_alpn_select_callback(|_ssl, alpn| {
        openssl::ssl::select_next_proto(tonic_tls::openssl::ALPN_H2_WIRE, alpn)
            .ok_or(openssl::ssl::AlpnError::NOACK)
    });
    // Don't require client to have a certificate
    acceptor.set_verify(openssl::ssl::SslVerifyMode::NONE);

    let tls_acceptor = acceptor.build();
    let tcp_incoming = tonic::transport::server::TcpIncoming::bind(address)?;
    let incoming = tonic_tls::openssl::TlsIncoming::new(tcp_incoming, tls_acceptor);

    Ok(incoming)
}
