mod env_vars;
mod grpc;

use grpc::StarknetCashierState;
use starknet_cashier::StarknetCashierServer;

use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    info!("Initializing starknet cashier...");

    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("starknet-cashier.env")
            .inspect_err(|e| tracing::error!("dotenvy initialization failed: {e}"));
    }

    let socket_addr = {
        let (_, _, _, socket_port) = env_vars::read_env_variables()?;
        format!("[::0]:{}", socket_port).parse()?
    };

    let state = StarknetCashierState::new().await?;

    let cashier_server_service = StarknetCashierServer::new(state);
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<StarknetCashierServer<StarknetCashierState>>()
        .await;

    info!("listening to new request on {}", socket_addr);

    tonic::transport::Server::builder()
        .add_service(cashier_server_service)
        .add_service(health_service)
        .serve(socket_addr)
        .await?;

    Ok(())
}
