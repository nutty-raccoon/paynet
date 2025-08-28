mod background_tasks;
mod commands;
mod errors;
mod migrations;
mod parse_asset_amount;

use commands::{
    add_node, check_wallet_exists, create_mint_quote, create_wads, get_currencies,
    get_nodes_balance, get_wad_history, init_wallet, receive_wads, redeem_quote,
    refresh_node_keysets, restore_wallet, set_price_provider_currency, sync_wads,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::time::SystemTime;
use std::{collections::HashSet, env, sync::Arc};
use tauri::{Listener, Manager, async_runtime};
use tokio::sync::RwLock;
use tonic::transport::Certificate;

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
                let host = env!("PRICE_PROVIDER_URL");
                let mut initial_assets = HashSet::new();
                if let Ok(conn) = pool.get() {
                    if let Ok(nodes_balances) = wallet::db::balance::get_for_all_nodes(&conn) {
                        for nb in nodes_balances {
                            for b in nb.balances {
                                let unit = if b.unit.eq_ignore_ascii_case("millistrk") {
                                    "strk".to_string()
                                } else {
                                    b.unit.to_lowercase()
                                };
                                initial_assets.insert(unit);
                            }
                        }
                    }
                }
                app.manage(AppState {
                    pool,
                    get_prices_config: Arc::new(RwLock::new(PriceConfig {
                        currency: "usd".to_string(),
                        assets: initial_assets,
                        url: host.to_string(),
                        status: Default::default(),
                    })),
                    #[cfg(feature = "tls-local-mkcert")]
                    tls_root_ca_cert: read_tls_root_ca_cert(),
                });
                let config = app.state::<AppState>().get_prices_config.clone();

                let app_thread = app.handle().clone();
                // Wait until the front is listening to start fetching prices
                app.once("front-ready", |_| {
                    async_runtime::spawn(start_price_fetcher(config, app_thread));
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
                get_currencies,
                check_wallet_exists,
                init_wallet,
                restore_wallet,
                set_price_provider_currency,
                get_wad_history,
                sync_wads,
            ])
    };

    if let Err(e) = app.run(tauri::generate_context!()) {
        // Use grep "tauri-app-run-error" to filter the startup error in logs
        log::error!("tauri-app-run-error: {e}");
        panic!("error while running tauri application: {e}");
    }
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
    get_prices_config: Arc<RwLock<PriceConfig>>,
    #[cfg(feature = "tls-local-mkcert")]
    tls_root_ca_cert: Certificate,
}

#[derive(Clone, Debug)]
pub struct PriceConfig {
    pub currency: String,
    pub assets: HashSet<String>,
    pub url: String,
    pub status: PriceSyncStatus,
}

#[derive(Debug, Clone, Default)]
pub enum PriceSyncStatus {
    #[default]
    NotSynced,
    Synced(SystemTime),
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
