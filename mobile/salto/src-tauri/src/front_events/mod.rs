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
