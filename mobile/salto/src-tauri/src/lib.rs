mod background_tasks;
mod commands;
mod errors;
mod migrations;
mod parse_asset_amount;

use commands::{
    add_node, check_wallet_exists, create_mint_quote, create_wads, get_currencies,
    get_nodes_balance, get_wad_history, init_wallet, price_provider_add_assets,
    price_provider_add_currencies, receive_wads, redeem_quote, restore_wallet, sync_wads,
    PriceResponce,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::{collections::HashSet, env, sync::Arc};
use tauri::{async_runtime, Manager};
use tokio::sync::RwLock;

use crate::background_tasks::start_price_fetcher;

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
                let host =
                    env::var("PRICE_PROVIDER").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
                app.manage(AppState {
                    pool,
                    get_prices_config: Arc::new(RwLock::new(PriceConfig {
                        currencies: HashSet::from(["usd".to_string()]),
                        assets: HashSet::new(),
                        url: host,
                    })),
                });
                let config = app.state::<AppState>().get_prices_config.clone();

                let app_thread = app.handle().clone();
                async_runtime::spawn(start_price_fetcher(config, app_thread));
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
                get_currencies,
                check_wallet_exists,
                init_wallet,
                restore_wallet,
                price_provider_add_assets,
                price_provider_add_currencies,
                get_wad_history,
                sync_wads,
            ])
    };

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Clone, Debug)]
pub struct PriceConfig {
    currencies: HashSet<String>,
    assets: HashSet<String>,
    url: String,
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
    get_prices_config: Arc<RwLock<PriceConfig>>,
}
