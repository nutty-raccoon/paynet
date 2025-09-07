use tauri::{AppHandle, Emitter, Error};

pub const BALANCE_INCREASE_EVENT: &str = "balance-increase";
pub const BALANCE_DECREASE_EVENT: &str = "balance-decrease";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceChange {
    pub node_id: u32,
    pub unit: String,
    pub amount: u64,
}

pub fn emit_balance_increase_event(app: &AppHandle, event: BalanceChange) -> Result<(), Error> {
    app.emit(BALANCE_INCREASE_EVENT, event)
}

pub fn emit_balance_decrease_event(app: &AppHandle, event: BalanceChange) -> Result<(), Error> {
    app.emit(BALANCE_DECREASE_EVENT, event)
}
