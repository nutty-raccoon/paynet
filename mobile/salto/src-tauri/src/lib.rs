mod background_tasks;
mod commands;
mod connection_cache;
mod errors;
mod front_events;
mod migrations;
mod quote_handler;

use commands::{
    add_node, check_wallet_exists, create_melt_quote, create_mint_quote, create_wads,
    get_currencies, get_nodes_balance, get_pending_quotes, get_wad_history, init_wallet,
    pay_melt_quote, pay_mint_quote, receive_wads, redeem_quote, refresh_node_keysets,
    restore_wallet, set_price_provider_currency, sync_wads,
};
use connection_cache::ConnectionCache;
use node_client::NodeClient;
use nuts::traits::Unit as UnitT;
use quote_handler::{QuoteHandlerEvent, start_syncing_quotes};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use starknet_types::Asset;
use std::{
    collections::HashSet,
    env,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tauri::{Listener, Manager, async_runtime};
use tokio::sync::{RwLock, mpsc};
use tonic::transport::{Certificate, Channel};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::background_tasks::start_price_fetcher;

// Value must be the same as the one configurated in tauri.conf.json["identifier"]
const SEED_PHRASE_MANAGER: wallet::wallet::keyring::SeedPhraseManager =
    wallet::wallet::keyring::SeedPhraseManager::new("com.salto.app");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = {
        #[cfg(target_os = "android")]
        android_keyring::set_android_keyring_credential_builder().unwrap();

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .compact(),
            )
            .with(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .init();

        let builder = tauri::Builder::default();

        #[cfg(target_os = "macos")]
        let builder = builder.plugin(tauri_plugin_macos_permissions::init());

        let builder = builder
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

                    app.manage(AppState {
                        pool,
                        web_app_url: web_app_url.to_string(),
                        get_prices_config: Arc::new(RwLock::new(PriceConfig {
                            currency: "usd".to_string(),
                            assets: initial_assets,
                            url: price_provider_url.to_string(),
                            status: Default::default(),
                        })),
                        #[cfg(feature = "tls-local-mkcert")]
                        tls_root_ca_cert: read_tls_root_ca_cert(),
                        quote_event_sender: tx,
                        connection_cache: connection_cache.clone(),
                    });

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
                set_price_provider_currency,
                get_wad_history,
                sync_wads,
                create_melt_quote,
                pay_melt_quote
            ])
    };

    if let Err(e) = app.run(tauri::generate_context!()) {
        // Use grep "tauri-app-run-error" to filter the startup error in logs
        tracing::error!("tauri-app-run-error: {e}");
        panic!("error while running tauri application: {e}");
    }
}

#[derive(Debug)]
struct AppState {
    pool: Pool<SqliteConnectionManager>,
    web_app_url: String,
    get_prices_config: Arc<RwLock<PriceConfig>>,
    #[cfg(feature = "tls-local-mkcert")]
    tls_root_ca_cert: Certificate,
    quote_event_sender: mpsc::Sender<QuoteHandlerEvent>,
    connection_cache: Arc<ConnectionCache>,
}

#[derive(Clone, Debug)]
pub struct PriceConfig {
    pub currency: String,
    pub assets: HashSet<Asset>,
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

    pub async fn get_node_client_connection(
        &self,
        node_id: u32,
    ) -> Result<NodeClient<Channel>, connection_cache::ConnectionCacheError> {
        self.connection_cache.get_or_create_client(node_id).await
    }
}

#[cfg(feature = "tls-local-mkcert")]
fn read_tls_root_ca_cert() -> Certificate {
    tonic::transport::Certificate::from_pem(include_bytes!("../certs/rootCA.pem"))
}
