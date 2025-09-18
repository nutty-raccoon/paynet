use std::collections::BTreeMap;

use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut01::{self, PublicKey, SetKeyPairs},
    nut02::{self},
    nut03::{SwapRequest, SwapResponse},
    nut04::{self, MintQuoteResponse, MintRequest, MintResponse},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::CheckStateResponse,
};
use thiserror::Error;

mod grpc_client;
mod proof_errors_handler;

pub use grpc_client::GrpcClient;
pub use proof_errors_handler::ProofErrorHandler;

use crate::proof_errors_handler::ProofError;

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
    #[error("invalid field format: '[' or ']' not found")]
    InvalidFormat,
    #[error("invalid index: {0}")]
    ParseError(#[from] std::num::ParseIntError),
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

#[async_trait::async_trait]
pub trait CashuClient: ProofErrorHandler + Send + Sync + Clone {
    async fn keysets(&mut self) -> Result<ClientKeysetsResponse, Error>;
    async fn keys(&mut self, keyset_id: Option<Vec<u8>>) -> Result<ClientKeysResponse, Error>;
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
    async fn restore(
        &mut self,
        outputs: Vec<BlindedMessage>,
    ) -> Result<ClientRestoreResponse, Error>;
}
