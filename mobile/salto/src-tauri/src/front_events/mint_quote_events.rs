use tauri::{AppHandle, Emitter, Error};

pub const NODE_PENDING_MINT_QUOTE_UPDATES: &str = "pending-mint-quote-updated";
pub const REMOVE_MINT_QUOTE_EVENT: &str = "remove-mint-quote";
pub const MINT_QUOTE_REDEEMED_EVENT: &str = "mint-quote-redeemed";
pub const MINT_QUOTE_PAID_EVENT: &str = "mint-quote-paid";
pub const MINT_QUOTE_CREATED_EVENT: &str = "mint-quote-created";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintQuoteCreatedEvent {
    pub node_id: u32,
    pub mint_quote: PendingMintQuoteData,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct MintQuotePaidEvent(pub MintQuoteIdentifier);

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct MintQuoteRedeemedEvent(pub MintQuoteIdentifier);

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct RemoveMintQuoteEvent(pub MintQuoteIdentifier);

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePendingMintQuotesStateUpdatesEvent {
    pub node_id: u32,
    pub unpaid: Vec<PendingMintQuoteData>,
    pub paid: Vec<PendingMintQuoteData>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingMintQuoteData {
    pub id: String,
    pub unit: String,
    pub amount: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintQuoteIdentifier {
    pub node_id: u32,
    pub quote_id: String,
}

pub fn emit_mint_quote_created_event(
    app: &AppHandle,
    event: MintQuoteCreatedEvent,
) -> Result<(), Error> {
    app.emit(MINT_QUOTE_CREATED_EVENT, event)
}

pub fn emit_mint_quote_paid_event(app: &AppHandle, event: MintQuotePaidEvent) -> Result<(), Error> {
    app.emit(MINT_QUOTE_PAID_EVENT, event)
}

pub fn emit_mint_quote_redeemed_event(
    app: &AppHandle,
    event: MintQuoteRedeemedEvent,
) -> Result<(), Error> {
    app.emit(MINT_QUOTE_REDEEMED_EVENT, event)
}

pub fn emit_remove_mint_quote_event(
    app: &AppHandle,
    event: RemoveMintQuoteEvent,
) -> Result<(), Error> {
    app.emit(REMOVE_MINT_QUOTE_EVENT, event)
}

pub fn emit_node_pending_mint_quotes_updates_event(
    app: &AppHandle,
    event: NodePendingMintQuotesStateUpdatesEvent,
) -> Result<(), Error> {
    app.emit(NODE_PENDING_MINT_QUOTE_UPDATES, event)
}
