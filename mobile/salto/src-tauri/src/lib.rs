mod commands;
mod errors;
mod migrations;

use commands::{add_node, create_mint_quote, get_nodes_balance, redeem_quote};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

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
            .invoke_handler(tauri::generate_handler![
                get_nodes_balance,
                add_node,
                create_mint_quote,
                redeem_quote,
            ])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
}
