use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut01::PublicKey,
    nut02::KeysetId,
    nut03::{SwapRequest, SwapResponse},
    nut04::{MintQuoteResponse, MintRequest, MintResponse},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::CheckStateResponse,
};
use proof_errors_handler::ProofError;
use thiserror::Error;

mod grpc_client;
mod proof_errors_handler;
mod rpc_client;

pub use grpc_client::GrpcClient;
pub use rpc_client::RpcClient;

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct ClientKeyset {
    pub id: Vec<u8>,
    pub unit: String,
    pub active: bool,
}

#[derive(Debug)]
pub struct ClientKeysetsResponse {
    pub keysets: Vec<ClientKeyset>,
}

#[derive(Debug)]
pub struct ClientKeysRequest {
    pub keyset_id: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct ClientKey {
    pub amount: Amount,
    pub publickey: PublicKey,
}

#[derive(Debug)]
pub struct ClientKeysetKeys {
    pub id: Vec<u8>,
    pub unit: String,
    pub active: bool,
    pub keys: Vec<ClientKey>,
}

pub struct ClientKeysResponse {
    pub keysets: Vec<ClientKeysetKeys>,
}

#[derive(Debug)]
pub struct ClientRestoreResponse {
    pub outputs: Vec<BlindedMessage>,
    pub signatures: Vec<BlindSignature>,
}

#[derive(Debug, thiserror::Error)]
pub enum CashuClientError {
    #[error("invalid proofs: {0:?}")]
    Proof(Vec<ProofError>),
    #[error("inactive keyset")]
    InactiveKeyset,
    #[error("quote not found")]
    QuoteNotFound,
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[async_trait::async_trait]
pub trait CashuClient: Send + Sync + Clone {
    type InnerError: std::error::Error;

    async fn keysets(&mut self) -> Result<ClientKeysetsResponse, CashuClientError>;
    async fn keys(
        &mut self,
        keyset_id: Option<KeysetId>,
    ) -> Result<ClientKeysResponse, CashuClientError>;
    async fn mint_quote(
        &mut self,
        req: ClientMintQuoteRequest,
    ) -> Result<MintQuoteResponse<String>, CashuClientError>;
    async fn mint(
        &mut self,
        req: MintRequest<String>,
        method: String,
    ) -> Result<MintResponse, CashuClientError>;
    async fn mint_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<MintQuoteResponse<String>, CashuClientError>;
    async fn swap(&mut self, req: SwapRequest) -> Result<SwapResponse, CashuClientError>;
    async fn melt_quote(
        &mut self,
        req: ClientMeltQuoteRequest,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError>;
    async fn melt(
        &mut self,
        method: String,
        req: MeltRequest<String>,
    ) -> Result<MeltResponse, CashuClientError>;
    async fn melt_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError>;
    async fn info(&mut self) -> Result<NodeInfoResponse, CashuClientError>;
    async fn check_state(
        &mut self,
        req: CheckStateRequest,
    ) -> Result<CheckStateResponse, CashuClientError>;
    async fn acknowledge(
        &mut self,
        path: String,
        request_hash: u64,
    ) -> Result<(), CashuClientError>;
    async fn restore(
        &mut self,
        outputs: Vec<BlindedMessage>,
    ) -> Result<ClientRestoreResponse, CashuClientError>;
}
