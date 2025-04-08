mod errors;
mod migrations;

use std::sync::Mutex;

use errors::GetNodesBalanceError;
use tauri::{Manager, State};
use tauri_plugin_log::{Target, TargetKind};
use wallet::db::balance::NodeBalances;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_nodes_balance(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<NodeBalances>, GetNodesBalanceError> {
    let state = state
        .lock()
        .map_err(|_| GetNodesBalanceError::StateMutexPoisoned)?;
    wallet::db::balance::get_for_all_nodes(&state.db_conn).map_err(GetNodesBalanceError::Rusqlite)
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
                let db_conn = rusqlite::Connection::open(db_path)?;

                // wallet::db::create_tables(&mut db_conn).expect("should run the migration successfully");
                app.manage(Mutex::new(AppState { db_conn }));

                Ok(())
            })
            .invoke_handler(tauri::generate_handler![get_nodes_balance])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppState {
    db_conn: rusqlite::Connection,
}
