mod app_state;
mod background_tasks;
mod commands;
mod errors;
mod front_events;
mod migrations;
mod quote_handler;

use app_state::{
    AppState, PriceConfig,
    connection_cache::{self, ConnectionCache},
};
use commands::{
    add_node, check_wallet_exists, create_melt_quote, create_mint_quote, create_wads, forget_node,
    get_currencies, get_nodes_balance, get_nodes_deposit_methods, get_pending_quotes,
    get_seed_phrase, get_wad_history, init_wallet, pay_melt_quote, pay_mint_quote, receive_wads,
    redeem_quote, refresh_node_keysets, restore_wallet, set_price_provider_currency, sync_wads,
};
use nuts::traits::Unit as UnitT;
use quote_handler::start_syncing_quotes;
use r2d2_sqlite::SqliteConnectionManager;
use std::{collections::HashSet, env, str::FromStr, sync::Arc, time::Duration};
use tauri::{Listener, Manager, async_runtime};
use tokio::sync::{Mutex, RwLock, mpsc};

use crate::background_tasks::start_price_fetcher;

// Value must be the same as the one configurated in tauri.conf.json["identifier"]
const SEED_PHRASE_MANAGER: wallet::wallet::keyring::SeedPhraseManager =
    wallet::wallet::keyring::SeedPhraseManager::new("com.salto.app");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = {
        #[cfg(target_os = "android")]
        android_keyring::set_android_keyring_credential_builder().unwrap();

        let builder = tauri::Builder::default();

        #[cfg(target_os = "macos")]
        let builder = builder.plugin(tauri_plugin_macos_permissions::init());
        #[cfg(any(target_os = "android", target_os = "ios"))]
        let builder = builder.plugin(tauri_plugin_biometric::init());

        let builder = builder
            .plugin(
                tauri_plugin_log::Builder::new()
                    .level(log::LevelFilter::Info)
                    .build(),
            )
            .plugin(tauri_plugin_os::init())
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_clipboard_manager::init());

        builder
            .setup(|app| {
                // Init db pool
                let pool = {
                    let mut db_path = app.handle().path().app_data_dir()?;
                    db_path.push("salto-wallet.sqlite3");

                    r2d2::Pool::new(SqliteConnectionManager::file(db_path))?
                };
                let app_handle = app.handle();

                let (tx, rx) = mpsc::channel(10);

                // Init State
                {
                    let price_provider_url = env!("PRICE_PROVIDER_URL");
                    let web_app_url = env!("WEB_APP_URL");

                    let mut initial_assets = HashSet::new();
                    if let Ok(conn) = pool.get() {
                        if let Ok(nodes_balances) = wallet::db::balance::get_for_all_nodes(&conn) {
                            for nb in nodes_balances {
                                for b in nb.balances {
                                    let unit = starknet_types::Unit::from_str(&b.unit)?;
                                    initial_assets.insert(unit.matching_asset());
                                }
                            }
                        }
                    }

                    // Initialize connection cache
                    let connection_cache = Arc::new(ConnectionCache::new(
                        pool.clone(),
                        Duration::from_secs(15 * 60), // 15 minute TTL
                        #[cfg(feature = "tls-local-mkcert")]
                        Some(read_tls_root_ca_cert()),
                        #[cfg(not(feature = "tls-local-mkcert"))]
                        None,
                    ));

                    let app_state = AppState::new(
                        pool,
                        web_app_url.to_string(),
                        Arc::new(RwLock::new(PriceConfig {
                            currency: "usd".to_string(),
                            assets: initial_assets,
                            url: price_provider_url.to_string(),
                            status: Default::default(),
                        })),
                        tx,
                        connection_cache.clone(),
                        Mutex::new(()),
                        #[cfg(feature = "tls-local-mkcert")]
                        read_tls_root_ca_cert(),
                    );
                    app.manage(app_state);

                    // Start cache cleanup background task
                    async_runtime::spawn(connection_cache::start_cache_cleanup_task(
                        connection_cache,
                    ));
                }

                let cloned_app_handle = app_handle.clone();
                let _handle = async_runtime::spawn(start_syncing_quotes(cloned_app_handle, rx));
                // Wait until the front is listening to start fetching prices
                let cloned_app_handle = app_handle.clone();
                app.once("front-ready", |_| {
                    async_runtime::spawn(start_price_fetcher(cloned_app_handle));
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
                pay_mint_quote,
                get_pending_quotes,
                redeem_quote,
                create_wads,
                receive_wads,
                get_currencies,
                check_wallet_exists,
                init_wallet,
                restore_wallet,
                get_seed_phrase,
                set_price_provider_currency,
                get_wad_history,
                sync_wads,
                create_melt_quote,
                pay_melt_quote,
                forget_node,
                get_nodes_deposit_methods,
            ])
    };

    if let Err(e) = app.run(tauri::generate_context!()) {
        // Use grep "tauri-app-run-error" to filter the startup error in logs
        tracing::error!("tauri-app-run-error: {e}");
        panic!("error while running tauri application: {e}");
    }
}

#[cfg(feature = "tls-local-mkcert")]
fn read_tls_root_ca_cert() -> tonic::transport::Certificate {
    tonic::transport::Certificate::from_pem(include_bytes!("../certs/rootCA.pem"))
}
