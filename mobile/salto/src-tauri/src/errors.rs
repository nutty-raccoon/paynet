#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("failed to get a connection from the pool: {0}")]
    DbPool(r2d2::Error),
    #[error("failed to interact with db: {0}")]
    Db(rusqlite::Error),
    #[error("cached connection error: {0}")]
    CachedConnection(#[from] crate::connection_cache::ConnectionCacheError),
    #[error("failed to push event in a the mint error channel")]
    QuoteHandlerChannel,
    #[error("failed wallet logic: {0}")]
    Wallet(#[from] wallet::errors::Error),
    #[error("quote {0} not found")]
    QuoteNotFound(String),
    #[error("failed to join tokio task: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}
