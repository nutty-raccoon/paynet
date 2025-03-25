use node::KeysetRotationServiceServer;
use std::net::SocketAddr;
use tonic_health::{pb::health_server::Health, server::HealthReporter};

use futures::TryFutureExt;
use node::NodeServer;
use nuts::QuoteTTLConfig;
use signer::SignerClient;
use sqlx::Postgres;
use starknet_cashier::StarknetCashierClient;
use starknet_types::Unit;
use tonic::transport::Channel;

use crate::{errors::ServiceError, grpc_service::GrpcState};

use super::Error;

pub async fn launch_tonic_server_task(
    pg_pool: sqlx::Pool<Postgres>,
    signer_client: SignerClient<Channel>,
    #[cfg(feature = "starknet")] starknet_cashier: StarknetCashierClient<Channel>,
    port: u16,
) -> Result<
    (
        SocketAddr,
        impl Future<Output = Result<(), crate::Error>>,
        HealthReporter,
    ),
    crate::Error,
> {
    let nuts_settings = super::nuts_settings::nuts_settings();
    let grpc_state = GrpcState::new(
        pg_pool,
        signer_client,
        nuts_settings,
        QuoteTTLConfig {
            mint_ttl: 3600,
            melt_ttl: 3600,
        },
        #[cfg(feature = "starknet")]
        starknet_cashier,
    );
    let address = format!("[::0]:{}", port)
        .parse()
        .map_err(|e| crate::Error::Init(Error::InvalidGrpcAddress(e)))?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();

    grpc_state
        .init_first_keysets(&[Unit::MilliStrk], 0, 32)
        .await?;
    let tonic_future = tonic::transport::Server::builder()
        .add_service(KeysetRotationServiceServer::new(grpc_state.clone()))
        .add_service(NodeServer::new(grpc_state))
        .add_service(health_service)
        .serve(address)
        .map_err(|e| crate::Error::Service(ServiceError::TonicTransport(e)));

    Ok((address, tonic_future, health_reporter))
}

pub async fn set_serving(health_reporter: HealthReporter) -> Result<(), crate::Error> {
    health_reporter
        .set_serving::<HealthServer<impl Health>>()
        .await;
    health_reporter.set_serving::<NodeServer<GrpcState>>().await;
    health_reporter
        .set_serving::<KeysetRotationServiceServer<GrpcState>>()
        .await;
}
