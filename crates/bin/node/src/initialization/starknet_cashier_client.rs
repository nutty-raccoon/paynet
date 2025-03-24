use starknet_cashier::{ConfigRequest, StarknetCashierClient};
use starknet_types::ChainId;
use tonic::transport::Channel;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to connect to cashier: {0}")]
    Connection(#[source] tonic::transport::Error),
    #[error("failed to get cashier config: {0}")]
    GetConfig(#[source] tonic::Status),
    #[error("node expected chain id '{0}' while cashier is using '{1}")]
    DifferentChainId(ChainId, String),
}

pub async fn connect_to_starknet_cashier(
    cashier_url: String,
    chain_id: ChainId,
) -> Result<StarknetCashierClient<Channel>, super::Error> {
    let mut starknet_cashier = starknet_cashier::StarknetCashierClient::connect(cashier_url)
        .await
        .map_err(Error::Connection)?;

    let config = starknet_cashier
        .config(tonic::Request::new(ConfigRequest {}))
        .await
        .map_err(Error::GetConfig)?
        .into_inner();
    if chain_id.as_ref() != &config.chain_id {
        Err(Error::DifferentChainId(chain_id, config.chain_id))?;
    }

    Ok(starknet_cashier)
}
