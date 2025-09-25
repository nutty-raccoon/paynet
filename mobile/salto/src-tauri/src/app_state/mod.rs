pub mod connection_cache;

use std::{collections::HashSet, sync::Arc, time::SystemTime};

use connection_cache::{ConnectionCache, NodeInfo};
use node_client::NodeClient;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use starknet_types::Asset;
use tokio::sync::{Mutex, RwLock, mpsc};
use tonic::transport::{Certificate, Channel};

use crate::quote_handler::QuoteHandlerEvent;

#[derive(Debug)]
pub struct AppState {
    pool: Pool<SqliteConnectionManager>,
    web_app_url: String,
    get_prices_config: Arc<RwLock<PriceConfig>>,
    quote_event_sender: mpsc::Sender<QuoteHandlerEvent>,
    connection_cache: Arc<ConnectionCache>,
    spend_proofs_lock: Mutex<()>,
    #[cfg(feature = "tls-local-mkcert")]
    tls_root_ca_cert: Certificate,
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
    pub fn new(
        pool: Pool<SqliteConnectionManager>,
        web_app_url: String,
        get_prices_config: Arc<RwLock<PriceConfig>>,
        quote_event_sender: mpsc::Sender<QuoteHandlerEvent>,
        connection_cache: Arc<ConnectionCache>,
        spend_proofs_lock: Mutex<()>,
        #[cfg(feature = "tls-local-mkcert")] tls_root_ca_cert: Certificate,
    ) -> Self {
        AppState {
            pool,
            web_app_url,
            get_prices_config,
            tls_root_ca_cert,
            quote_event_sender,
            connection_cache,
            spend_proofs_lock,
        }
    }

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

    pub async fn get_node_info(
        &self,
        node_id: u32,
    ) -> Result<Option<NodeInfo>, connection_cache::ConnectionCacheError> {
        let _ = self.connection_cache.get_or_create_client(node_id).await;

        Ok()
    }
}
