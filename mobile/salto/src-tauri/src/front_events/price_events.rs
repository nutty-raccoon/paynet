use tauri::{AppHandle, Emitter, Error};

pub const NEW_PRICE_EVENT: &str = "new-price";
pub const OUT_OF_SYNC_PRICE_EVENT: &str = "out-of-sync-price";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPriceEvent {
    pub symbol: String,
    pub value: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OutOfSyncPriceEvent;

pub fn emit_new_price_event(app: &AppHandle, events: Vec<NewPriceEvent>) -> Result<(), Error> {
    app.emit(NEW_PRICE_EVENT, events)
}

pub fn emit_out_of_sync_price_event(
    app: &AppHandle,
    event: OutOfSyncPriceEvent,
) -> Result<(), Error> {
    app.emit(OUT_OF_SYNC_PRICE_EVENT, event)
}
