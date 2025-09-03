pub const MINT_QUOTE_CREATED_EVENT: &str = "mint-quote-created";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintQuoteCreatedEvent {
    pub node_id: u32,
    pub mint_quote: PendingMintQuoteData,
}

pub const MINT_QUOTE_PAID_EVENT: &str = "mint-quote-paid";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct MintQuotePaidEvent(pub MintQuoteIdentifier);

pub const MINT_QUOTE_REDEEMED_EVENT: &str = "mint-quote-redeemed";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct MintQuoteRedeemedEvent(pub MintQuoteIdentifier);

pub const REMOVE_MINT_QUOTE_EVENT: &str = "remove-mint-quote";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct RemoveMintQuoteEvent(pub MintQuoteIdentifier);

pub const BALANCE_INCREASE_EVENT: &str = "balance-increase";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    pub node_id: u32,
    pub unit: String,
    pub amount: u64,
}

pub const NODE_PENDING_MINT_QUOTE_UPDATES: &str = "pending-mint-quote-updated";

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
