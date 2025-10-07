pub mod connection_cache;

use std::{collections::HashSet, sync::Arc, time::SystemTime};

use connection_cache::{ConnectionCache, NodeInfo};
use futures::future::try_join_all;
use node_client::NodeClient;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use starknet_types::Asset;
use tokio::sync::{Mutex, MutexGuard, RwLock, mpsc};
use tonic::transport::{Certificate, Channel};
use tracing::error;

use crate::{errors::CommonError, quote_handler::QuoteHandlerEvent};

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
            quote_event_sender,
            connection_cache,
            spend_proofs_lock,
            #[cfg(feature = "tls-local-mkcert")]
            tls_root_ca_cert,
        }
    }

    #[cfg(feature = "tls-local-mkcert")]
    pub fn opt_root_ca_cert(&self) -> Option<Certificate> {
        Some(self.tls_root_ca_cert.clone())
    }

    #[cfg(not(feature = "tls-local-mkcert"))]
    pub fn opt_root_ca_cert(&self) -> Option<Certificate> {
        None
    }

    pub async fn get_node_client_connection(
        &self,
        node_id: u32,
    ) -> Result<NodeClient<Channel>, connection_cache::ConnectionCacheError> {
        self.connection_cache.get_or_create_client(node_id).await
    }

    pub async fn get_nodes_info(
        &self,
        node_ids: Vec<u32>,
    ) -> Result<Vec<(u32, Option<NodeInfo>)>, CommonError> {
        let mut handles = Vec::with_capacity(node_ids.len());

        for node_id in node_ids {
            let cloned_conn_cache = self.connection_cache.clone();
            handles.push(tokio::spawn(async move {
                match cloned_conn_cache.get_node_info(node_id).await {
                    Ok(s) => (node_id, s),
                    Err(e) => {
                        error!("failed to get node info for node {node_id}: {e}");
                        (node_id, None)
                    }
                }
            }));
        }

        // TODO: this means the frontend will only get the data when the last node answered
        // An optimization would be to send events to the front end as the responses arrive
        let infos = try_join_all(handles)
            .await
            .map_err(CommonError::TokioJoin)?;

        Ok(infos)
    }

    pub fn quote_event_sender(&self) -> mpsc::Sender<QuoteHandlerEvent> {
        self.quote_event_sender.clone()
    }

    pub fn web_app_url(&self) -> &str {
        &self.web_app_url
    }

    pub fn pool(&self) -> &Pool<SqliteConnectionManager> {
        &self.pool
    }

    pub fn get_prices_config(&self) -> Arc<RwLock<PriceConfig>> {
        self.get_prices_config.clone()
    }

    pub async fn lock_proof_spending(&self) -> MutexGuard<'_, ()> {
        self.spend_proofs_lock.lock().await
    }
}
