use super::*;
use async_trait::async_trait;
use node_client::{self as grpc, NodeClient};
use nuts::{
    Amount,
    nut00::{BlindedMessage, Proof},
    nut04, nut05,
    nut06::NodeInfo,
};
use tonic::transport::Channel;
use uuid::Uuid;

#[derive(Clone)]
pub struct GrpcTransport {
    inner: NodeClient<Channel>,
}

impl GrpcTransport {
    pub async fn connect(url: &str, timeout_ms: u64) -> Result<Self, NodeError> {
        let endpoint = tonic::transport::Endpoint::from_shared(url.to_string())
            .map_err(|e| NodeError::Transport(e.to_string()))?
            .timeout(std::time::Duration::from_millis(timeout_ms));
        let chan = endpoint
            .connect()
            .await
            .map_err(|e| NodeError::Transport(e.to_string()))?;
        Ok(Self {
            inner: NodeClient::new(chan),
        })
    }
}
