mod commands;
mod errors;
mod migrations;
mod parse_asset_amount;

use commands::{
    add_node, check_wallet_exists, create_mint_quote, create_wads, get_nodes_balance, init_wallet,
    receive_wads, redeem_quote, restore_wallet,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = {
        let builder = tauri::Builder::default();

        let builder = builder
            .plugin(tauri_plugin_log::Builder::new().build())
            .plugin(tauri_plugin_os::init())
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_clipboard_manager::init());

        builder
            .setup(|app| {
                let db_path = app
                    .handle()
                    .path()
                    .app_data_dir()
                    .map(|mut dp| {
                        dp.push("salto-wallet.sqlite3");
                        dp
                    })
                    .expect("dirs::data_dir should map to a valid path on this machine");
                let manager = SqliteConnectionManager::file(db_path);
                let pool = r2d2::Pool::new(manager)?;
                app.manage(AppState { pool });
                Ok(())
            })
            .plugin(
                tauri_plugin_sql::Builder::default()
                    .add_migrations("sqlite:salto-wallet.sqlite3", migrations::migrations())
                    .build(),
            )
            .invoke_handler(tauri::generate_handler![
                get_nodes_balance,
                add_node,
                create_mint_quote,
                redeem_quote,
                create_wads,
                receive_wads,
                check_wallet_exists,
                init_wallet,
                restore_wallet,
            ])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
}
