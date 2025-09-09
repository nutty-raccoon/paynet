use tauri::Emitter;
use tracing::instrument;

pub mod price_events;
pub mod wad_events;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingQuoteData {
    pub id: String,
    pub unit: String,
    pub amount: u64,
}

// Trigger polling events
const TRIGGER_BALANCE_POLL_EVENT: &str = "trigger-balance-poll";
const TRIGGER_PENDING_QUOTE_POLL_EVENT: &str = "trigger-pending-quote-poll";

// Helper functions to emit trigger events
#[instrument(skip(app))]
pub fn emit_trigger_balance_poll(app: &tauri::AppHandle) -> Result<(), tauri::Error> {
    app.emit(TRIGGER_BALANCE_POLL_EVENT, ())
}

#[instrument(skip(app))]
pub fn emit_trigger_pending_quote_poll(app: &tauri::AppHandle) -> Result<(), tauri::Error> {
    app.emit(TRIGGER_PENDING_QUOTE_POLL_EVENT, ())
}
