use tauri::{AppHandle, Emitter, Error};

pub const WAD_STATUS_UPDATED_EVENT: &str = "wad-status-updated";
pub const SYNC_WAD_ERROR_EVENT: &str = "sync-wad-error";

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WadStatusUpdatedEvent {
    pub wad_id: uuid::Uuid,
    pub new_status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncWadErrorEvent {
    pub wad_id: uuid::Uuid,
    pub error: String,
}

pub fn emit_wad_status_updated_event(
    app: &AppHandle,
    event: WadStatusUpdatedEvent,
) -> Result<(), Error> {
    app.emit(WAD_STATUS_UPDATED_EVENT, event)
}

pub fn emit_sync_wad_error_event(app: &AppHandle, event: SyncWadErrorEvent) -> Result<(), Error> {
    app.emit(SYNC_WAD_ERROR_EVENT, event)
}
