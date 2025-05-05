#[cfg(feature = "keyset-rotation")]
use node::KeysetRotationServiceServer;
use std::net::SocketAddr;

use futures::TryFutureExt;
use node::NodeServer;
use nuts::QuoteTTLConfig;
use signer::SignerClient;
use sqlx::Postgres;
use starknet_types::Unit;
use tonic::transport::Channel;

use crate::{grpc_service::GrpcState, liquidity_sources::LiquiditySources};

use super::Error;

pub async fn launch_tonic_server_task(
    pg_pool: sqlx::Pool<Postgres>,
    signer_client: SignerClient<Channel>,
    liquidity_sources: LiquiditySources,
    port: u16,
) -> Result<(SocketAddr, impl Future<Output = Result<(), crate::Error>>), crate::Error> {
    let nuts_settings = super::nuts_settings::nuts_settings();
    let grpc_state = GrpcState::new(
        pg_pool,
        signer_client,
        nuts_settings,
        QuoteTTLConfig {
            mint_ttl: 3600,
            melt_ttl: 3600,
        },
        liquidity_sources,
    );
    let address = format!("[::0]:{}", port)
        .parse()
        .map_err(|e| crate::Error::Init(Error::InvalidGrpcAddress(e)))?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<NodeServer<GrpcState>>().await;
    #[cfg(feature = "keyset-rotation")]
    health_reporter
        .set_serving::<KeysetRotationServiceServer<GrpcState>>()
        .await;

    grpc_state
        .init_first_keysets(&[Unit::MilliStrk], 0, 32)
        .await?;
    let mut tonic_server = tonic::transport::Server::builder();
    #[cfg(feature = "keyset-rotation")]
    let tonic_server =
        tonic_server.add_service(KeysetRotationServiceServer::new(grpc_state.clone()));
    let tonic_future = tonic_server
        .add_service(NodeServer::new(grpc_state))
        .add_service(health_service)
        .serve(address)
        .map_err(crate::Error::Tonic);

    Ok((address, tonic_future))
}
