use nuts::{
    Amount, nut01, nut02,
    nut03::{SwapRequest, SwapResponse},
    nut04::{self, MintQuoteResponse, MintRequest, MintResponse},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::CheckStateResponse,
};
use thiserror::Error;

mod grpc_client;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Grpc(#[from] tonic::Status),
    #[error("Invalid State in {method}")]
    InvalidState { method: String },
    #[error(transparent)]
    KeysetId(nut02::Error),
    #[error(transparent)]
    PublicKey(nut01::Error),
    #[error(transparent)]
    Method(nut04::Error),
}

#[derive(Debug)]
pub struct ClientMintQuoteRequest {
    pub method: String,
    pub amount: u64,
    pub unit: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct ClientMeltQuoteRequest {
    pub method: String,
    pub request: String,
    pub unit: String,
}

#[derive(Debug)]
pub struct ClientMeltQuoteResponse {
    pub quote: String,
    pub amount: Amount,
    pub unit: String,
    pub state: MeltQuoteState,
    pub expiry: u64,
    pub transfer_ids: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CheckStateRequest {
    pub ys: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct NodeInfoResponse {
    pub info: String,
}

#[async_trait::async_trait]
pub trait CashuClient: Send + Sync + Clone {
    async fn mint_quote(
        &mut self,
        req: ClientMintQuoteRequest,
    ) -> Result<MintQuoteResponse<String>, Error>;
    async fn mint(
        &mut self,
        req: MintRequest<String>,
        method: String,
    ) -> Result<MintResponse, Error>;
    async fn mint_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<MintQuoteResponse<String>, Error>;
    async fn swap(&mut self, req: SwapRequest) -> Result<SwapResponse, Error>;
    async fn melt_quote(
        &mut self,
        req: ClientMeltQuoteRequest,
    ) -> Result<ClientMeltQuoteResponse, Error>;
    async fn melt(
        &mut self,
        method: String,
        req: MeltRequest<String>,
    ) -> Result<MeltResponse, Error>;
    async fn melt_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<ClientMeltQuoteResponse, Error>;
    async fn info(&mut self) -> Result<NodeInfoResponse, Error>;
    async fn check_state(&mut self, req: CheckStateRequest) -> Result<CheckStateResponse, Error>;
    async fn acknowledge(&mut self, path: String, request_hash: u64) -> Result<(), Error>;
}
