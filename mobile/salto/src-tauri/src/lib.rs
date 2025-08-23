mod commands;
mod errors;
mod migrations;
mod parse_asset_amount;

use commands::{
    add_node, check_wallet_exists, create_mint_quote, create_wads, get_nodes_balance,
    get_wad_history, init_wallet, receive_wads, redeem_quote, refresh_node_keysets, restore_wallet,
    sync_wads,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::Manager;
use tonic::transport::Certificate;

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

                app.manage(AppState {
                    pool,
                    #[cfg(feature = "tls-local-mkcert")]
                    tls_root_ca_cert: read_tls_root_ca_cert(),
                });
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
                refresh_node_keysets,
                create_mint_quote,
                redeem_quote,
                create_wads,
                receive_wads,
                check_wallet_exists,
                init_wallet,
                restore_wallet,
                get_wad_history,
                sync_wads,
            ])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
    #[cfg(feature = "tls-local-mkcert")]
    tls_root_ca_cert: Certificate,
}

impl AppState {
    #[cfg(feature = "tls-local-mkcert")]
    fn opt_root_ca_cert(&self) -> Option<Certificate> {
        Some(self.tls_root_ca_cert.clone())
    }

    #[cfg(not(feature = "tls-local-mkcert"))]
    fn opt_root_ca_cert(&self) -> Option<Certificate> {
        None
    }
}

#[cfg(feature = "tls-local-mkcert")]
fn read_tls_root_ca_cert() -> Certificate {
    tonic::transport::Certificate::from_pem(include_bytes!("../certs/rootCA.pem"))
}
