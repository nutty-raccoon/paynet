use crate::AppState;
use starknet_types::Unit;
use tauri::{AppHandle, State, Emitter};
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WadHistoryItem {
    pub id: String,
    pub wad_type: String,
    pub status: String,
    pub total_amount_json: String,
    pub memo: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum GetWadHistoryError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl serde::Serialize for GetWadHistoryError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn get_wad_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<WadHistoryItem>, GetWadHistoryError> {
    let db_conn = state.pool.get()?;
    let wad_records = wallet::db::wad::get_recent_wads(&db_conn, limit.unwrap_or(20))?;

    let mut history_items = Vec::with_capacity(wad_records.len());
    for record in wad_records {
        let total_amount_json = serde_json::to_string(
            &wallet::db::wad::get_amounts_by_id::<Unit>(&db_conn, record.id)?,
        )?;

        history_items.push(WadHistoryItem {
            id: record.id.to_string(),
            wad_type: record.wad_type.to_string(),
            status: record.status.to_string(),
            total_amount_json,
            memo: record.memo,
            created_at: record.created_at,
            modified_at: record.modified_at,
        })
    }

    Ok(history_items)
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WadStatusUpdate {
    pub wad_id: Uuid,
    pub new_status: String,
}

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub wad_id: Uuid,
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncWadsError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl serde::Serialize for SyncWadsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn sync_wads(app: AppHandle, state: State<'_, AppState>) -> Result<(), SyncWadsError> {
    // Use the lib wallet function instead of duplicating code
    let wad_results = wallet::sync::sync_pending_wads(state.pool.clone()).await?;

    // Emit events for UI updates
    for result in wad_results {
        match result.result {
            Ok(None) => {}
            Ok(Some(status)) => {
                app.emit(
                    "wad-status-updated",
                    WadStatusUpdate {
                        wad_id: result.wad_id,
                        new_status: status.to_string(),
                    },
                )?;
            }
            Err(e) => {
                app.emit(
                    "sync-wad-error",
                    SyncError {
                        wad_id: result.wad_id,
                        error: e.to_string(),
                    },
                )?;
            }
        }
    }

    Ok(())
}
