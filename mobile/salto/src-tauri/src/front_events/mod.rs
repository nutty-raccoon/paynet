use tauri::{AppHandle, Emitter};

pub mod balance_events;
pub mod melt_quote_events;
pub mod mint_quote_events;
pub mod price_events;
pub mod wad_events;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QuoteType {
    Mint,
    Melt,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingQuoteData {
    pub id: String,
    pub unit: String,
    pub amount: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintPendingQuotes {
    pub unpaid: Vec<PendingQuoteData>,
    pub paid: Vec<PendingQuoteData>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeltPendingQuotes {
    pub unpaid: Vec<PendingQuoteData>,
    pub pending: Vec<PendingQuoteData>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct NodePendingQuotesStateUpdatesEvent {
    pub r#type: QuoteType,
    pub node_id: u32,
    pub mint: Option<MintPendingQuotes>,
    pub melt: Option<MeltPendingQuotes>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum QuoteEventData {
    #[serde(rename_all = "camelCase")]
    Created { quote: PendingQuoteData },
    #[serde(rename_all = "camelCase")]
    Identifier { quote_id: String },
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteEvent {
    pub r#type: String,
    pub node_id: u32,
    #[serde(flatten)]
    pub data: QuoteEventData,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteIdentifier {
    pub node_id: u32,
    pub quote_id: String,
}

const NODE_PENDING_QUOTES_UPDATES: &str = "pending-quotes-updated";
const QUOTE_EVENT: &str = "quote";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedQuoteEvent {
    pub r#type: String,
    pub quote_type: QuoteType,
    pub node_id: u32,
    #[serde(flatten)]
    pub data: QuoteEventData,
}

pub fn emit_node_pending_mint_quotes_updates_event(
    app: &AppHandle,
    node_id: u32,
    quotes: MintPendingQuotes,
) -> Result<(), tauri::Error> {
    app.emit(
        NODE_PENDING_QUOTES_UPDATES,
        NodePendingQuotesStateUpdatesEvent {
            r#type: QuoteType::Mint,
            node_id,
            mint: Some(quotes),
            melt: None,
        },
    )
}

pub fn emit_node_pending_melt_quotes_updates_event(
    app: &AppHandle,
    node_id: u32,
    quotes: MeltPendingQuotes,
) -> Result<(), tauri::Error> {
    app.emit(
        NODE_PENDING_QUOTES_UPDATES,
        NodePendingQuotesStateUpdatesEvent {
            r#type: QuoteType::Melt,
            node_id,
            mint: None,
            melt: Some(quotes),
        },
    )
}
