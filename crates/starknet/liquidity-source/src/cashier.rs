use starknet_cashier::{ConfigRequest, StarknetCashierClient};
use starknet_types::ChainId;
use tonic::transport::Channel;
use tracing::Level;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to connect to cashier: {0}")]
    Connection(#[from] tonic::transport::Error),
    #[error("failed to get cashier config: {0}")]
    GetConfig(#[source] tonic::Status),
    #[error("node expected chain id '{0}' while cashier is using '{1}")]
    DifferentChainId(ChainId, String),
    #[error("invalid cashier uri: {0}")]
    Uri(#[from] http::uri::InvalidUri),
}

pub async fn connect(
    cashier_url: String,
    chain_id: &ChainId,
) -> Result<StarknetCashierClient<tower_otel::trace::Grpc<Channel>>, Error> {
    let channel = Channel::builder(cashier_url.parse()?).connect().await?;
    let channel = tower::ServiceBuilder::new()
        .layer(tower_otel::trace::GrpcLayer::client(Level::INFO))
        .service(channel);

    let mut starknet_cashier = starknet_cashier::StarknetCashierClient::new(channel);

    let config = starknet_cashier
        .config(tonic::Request::new(ConfigRequest {}))
        .await
        .map_err(Error::GetConfig)?
        .into_inner();
    if chain_id.as_str() != config.chain_id {
        Err(Error::DifferentChainId(chain_id.clone(), config.chain_id))?;
    }

    Ok(starknet_cashier)
}
