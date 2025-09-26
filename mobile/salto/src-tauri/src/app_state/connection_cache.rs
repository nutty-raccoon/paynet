use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use node_client::{GetNodeInfoRequest, NodeClient};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::sync::RwLock;
use tonic::transport::{Certificate, Channel};
use tracing::info;
use wallet::{ConnectToNodeError, connect_to_node};

pub type NodeInfo = nuts::nut06::NodeInfo<String, String, serde_json::Value>;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionCacheError {
    #[error("failed to get database connection: {0}")]
    DatabasePool(#[from] r2d2::Error),
    #[error("failed to query database: {0}")]
    DatabaseQuery(#[from] rusqlite::Error),
    #[error("node {0} not found in database")]
    NodeNotFound(u32),
    #[error("failed to connect to node: {0}")]
    Connection(#[from] ConnectToNodeError),
}

#[derive(Debug, Clone)]
struct CachedConnection {
    client: NodeClient<Channel>,
    info: Option<NodeInfo>,
    created_at: SystemTime,
}

impl CachedConnection {
    async fn new(mut client: NodeClient<Channel>) -> Self {
        let opt_node_info = match client.get_node_info(GetNodeInfoRequest {}).await {
            Ok(r) => serde_json::from_str(&r.into_inner().info).ok(),
            Err(_) => None,
        };

        Self {
            client,
            info: opt_node_info,
            created_at: SystemTime::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.created_at)
            .map(|age| age > ttl)
            .unwrap_or(true)
    }
}

#[derive(Debug)]
pub struct ConnectionCache {
    cache: Arc<RwLock<HashMap<u32, CachedConnection>>>,
    pool: Pool<SqliteConnectionManager>,
    ttl: Duration,
    opt_root_ca_cert: Option<Certificate>,
}

impl ConnectionCache {
    pub fn new(
        pool: Pool<SqliteConnectionManager>,
        ttl: Duration,
        opt_root_ca_cert: Option<Certificate>,
    ) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            pool,
            ttl,
            opt_root_ca_cert,
        }
    }

    pub async fn get_or_create_client(
        &self,
        node_id: u32,
    ) -> Result<NodeClient<Channel>, ConnectionCacheError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&node_id) {
                if !cached.is_expired(self.ttl) {
                    return Ok(cached.client.clone());
                }
            }
        }

        // Get node URL from database
        let node_url = {
            let db_conn = self.pool.get()?;
            wallet::db::node::get_url_by_id(&db_conn, node_id)?
                .ok_or(ConnectionCacheError::NodeNotFound(node_id))?
        };

        // Create new connection
        let client = connect_to_node(&node_url, self.opt_root_ca_cert.clone()).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            let cached_connection = CachedConnection::new(client.clone()).await;
            cache.insert(node_id, cached_connection);
        }

        Ok(client)
    }

    pub async fn get_node_info(
        &self,
        node_id: u32,
    ) -> Result<Option<NodeInfo>, ConnectionCacheError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&node_id) {
                if !cached.is_expired(self.ttl) {
                    return Ok(cached.info.clone());
                }
            }
        }

        // Get node URL from database
        let node_url = {
            let db_conn = self.pool.get()?;
            wallet::db::node::get_url_by_id(&db_conn, node_id)?
                .ok_or(ConnectionCacheError::NodeNotFound(node_id))?
        };

        // Update cache
        let info = {
            // Create new connection
            let client = connect_to_node(&node_url, self.opt_root_ca_cert.clone()).await?;

            let mut cache = self.cache.write().await;
            let cached_connection = CachedConnection::new(client.clone()).await;
            let info = cached_connection.info.clone();
            cache.insert(node_id, cached_connection);

            info
        };

        Ok(info)
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;

        cache.retain(|_node_id, connection| !connection.is_expired(self.ttl));
    }

    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

pub async fn start_cache_cleanup_task(cache: Arc<ConnectionCache>) {
    let mut interval = tokio::time::interval(Duration::from_secs(5 * 60)); // 5 minutes

    loop {
        interval.tick().await;

        cache.cleanup_expired().await;

        // Log cache statistics periodically
        let size = cache.cache_size().await;
        info!("Connection cache contains {} active entries", size);
    }
}
