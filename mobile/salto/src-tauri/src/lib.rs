mod errors;
mod migrations;

use std::str::FromStr;

use errors::{AddNodeError, GetNodesBalanceError};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::{Manager, State};
use tauri_plugin_log::{Target, TargetKind};
use wallet::{db::balance::NodeBalances, types::NodeUrl};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_nodes_balance(
    state: State<'_, AppState>,
) -> Result<Vec<NodeBalances>, GetNodesBalanceError> {
    {
        let db_lock = state.pool.get()?;
        wallet::db::balance::get_for_all_nodes(&db_lock)
    }
    .map_err(GetNodesBalanceError::Rusqlite)
}

#[tauri::command]
async fn add_node(
    state: State<'_, AppState>,
    node_url: String,
) -> Result<(u32, bool), AddNodeError> {
    let node_url = NodeUrl::from_str(&node_url)?;
    {
        let (_client, id, is_new) = wallet::register_node(state.pool.clone(), node_url).await?;
        Ok((id, is_new))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = {
        let db_path = dirs::data_local_dir()
            .map(|mut dp| {
                dp.push("salto-wallet.sqlite3");
                dp
            })
            .expect("dirs::data_dir should map to a valid path on this machine");
        let db_url = format!(
            "sqlite:{}",
            db_path
                .as_path()
                .to_str()
                .expect("db url should be a valid string")
        );

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(
                tauri_plugin_log::Builder::new()
                    .target(Target::new(TargetKind::Stdout))
                    .level(log::LevelFilter::Info)
                    .build(),
            )
            .plugin(
                tauri_plugin_sql::Builder::default()
                    .add_migrations(&db_url, migrations::migrations())
                    .build(),
            )
            .setup(|app| {
                let manager = SqliteConnectionManager::file(db_path);
                let pool = r2d2::Pool::new(manager)?;

                app.manage(AppState { pool });

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![get_nodes_balance, add_node])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
}
