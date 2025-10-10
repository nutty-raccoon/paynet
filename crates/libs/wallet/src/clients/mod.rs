use nuts::{
    Amount,
    nut00::{BlindedMessage, Proof},
    nut03::SwapResponse,
    nut04::MintQuoteState as NutMintQuoteState,
    nut05::MeltQuoteState as NutMeltQuoteState,
    nut06::NodeInfo,
};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("conversion error: {0}")]
    Conversion(String),
}

#[derive(Debug, Clone)]
pub struct MintQuote {
    pub quote: Uuid,
    pub request: String,
    pub state: NutMintQuoteState,
    pub expiry: u64,
}

#[derive(Debug, Clone)]
pub struct MeltQuote {
    pub quote: Uuid,
    pub unit: String,
    pub amount: Amount,
    pub state: NutMeltQuoteState,
    pub expiry: u64,
    pub transfer_ids: Vec<String>,
}

pub trait NodeApi: Send + Sync + Clone {
    async fn keysets(&mut self) -> Result<Vec<(Vec<u8>, String, bool)>, NodeError>;
    async fn keys_for(
        &mut self,
        keyset_id: Option<Vec<u8>>,
    ) -> Result<Vec<(Vec<u8>, Vec<(u64, String)>)>, NodeError>;

    async fn swap(
        &mut self,
        inputs: Vec<Proof>,
        outputs: Vec<BlindedMessage>,
    ) -> Result<SwapResponse, NodeError>;

    async fn mint_quote(
        &mut self,
        method: String,
        amount: Amount,
        unit: String,
    ) -> Result<MintQuote, NodeError>;
    async fn mint(
        &mut self,
        method: String,
        quote: Uuid,
        outputs: Vec<BlindedMessage>,
    ) -> Result<Vec<(Amount, Vec<u8>, Vec<u8>)>, NodeError>; // signatures (amount, keyset_id, blind_sig)
    async fn mint_quote_state(
        &mut self,
        method: String,
        quote: Uuid,
    ) -> Result<MintQuote, NodeError>;

    async fn melt_quote(
        &mut self,
        method: String,
        unit: String,
        request: String,
    ) -> Result<MeltQuote, NodeError>;
    async fn melt(
        &mut self,
        method: String,
        quote: Uuid,
        inputs: Vec<Proof>,
    ) -> Result<(NutMeltQuoteState, Vec<String>), NodeError>;
    async fn melt_quote_state(
        &mut self,
        method: String,
        quote: Uuid,
    ) -> Result<MeltQuote, NodeError>;

    async fn check_state(&mut self, ys: Vec<Vec<u8>>) -> Result<Vec<(Vec<u8>, String)>, NodeError>; // (Y, state)
    async fn restore(
        &mut self,
        outputs: Vec<BlindedMessage>,
    ) -> Result<(Vec<BlindedMessage>, Vec<(Amount, Vec<u8>, Vec<u8>)>), NodeError>;

    async fn info(&mut self) -> Result<NodeInfo<Method, String, Value>, NodeError>;
    async fn acknowledge(&mut self, path: String, request_hash: u64) -> Result<(), NodeError>;
}
