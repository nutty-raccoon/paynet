mod env_vars;
mod grpc;

use std::net::SocketAddr;

use grpc::StarknetCashierState;
use starknet_cashier::StarknetCashierServer;

use tonic::service::LayerExt;
use tower::ServiceBuilder;
use tracing::trace;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    open_telemetry_tracing::init(PKG_NAME, PKG_VERSION);

    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("starknet-cashier.env")
            .inspect_err(|e| tracing::error!("dotenvy initialization failed: {e}"));
    }

    let socket_addr: SocketAddr = {
        let (_, _, _, socket_port) = env_vars::read_env_variables()?;
        format!("[::0]:{}", socket_port).parse()?
    };

    let state = StarknetCashierState::new().await?;

    let cashier_server_service = ServiceBuilder::new()
        .layer(tower_otel::trace::GrpcLayer::server(tracing::Level::INFO))
        .named_layer(StarknetCashierServer::new(state));
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<StarknetCashierServer<StarknetCashierState>>()
        .await;

    trace!(name: "grpc-listen", port = socket_addr.port());

    tonic::transport::Server::builder()
        .add_service(cashier_server_service)
        .add_service(health_service)
        .serve(socket_addr)
        .await?;

    Ok(())
}
