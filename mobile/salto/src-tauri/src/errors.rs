#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("failed to get a connection from the pool: {0}")]
    DbPool(r2d2::Error),
    #[error("failed to interact with db: {0}")]
    Db(rusqlite::Error),
    #[error("cached connection error: {0}")]
    CachedConnection(#[from] crate::connection_cache::ConnectionCacheError),
    #[error("failed to push event in a the mint error channel")]
    MintQuoteChannel,
    #[error("failed wallet logic: {0}")]
    Wallet(wallet::errors::Error),
    #[error("failed to emit tauri event: {0}")]
    EmitTauriEvent(tauri::Error),
    #[error("quote {0} not found")]
    QuoteNotFound(String),
}
