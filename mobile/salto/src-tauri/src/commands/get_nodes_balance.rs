use tauri::State;
use wallet::db::balance::NodeData;

use crate::AppState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub fn get_nodes_balance(state: State<'_, AppState>) -> Result<Vec<NodeData>, Error> {
    let db_conn = state.pool.get()?;
    let nodes_balances = wallet::db::balance::get_for_all_nodes(&db_conn)?;
    Ok(nodes_balances)
}
