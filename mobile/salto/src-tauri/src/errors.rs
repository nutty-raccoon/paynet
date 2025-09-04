#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("failed to get a connection from the pool: {0}")]
    DbPool(r2d2::Error),
    #[error("failed to interact with db: {0}")]
    Db(rusqlite::Error),
    #[error("failed to create node client: {0}")]
    CreateNodeClient(wallet::ConnectToNodeError),
    #[error("failed to push event in a the mint error channel")]
    MintQuoteChannel,
    #[error("failed wallet logic: {0}")]
    Wallet(wallet::errors::Error),
    #[error("failed to emit tauri event: {0}")]
    EmitTauriEvent(tauri::Error),
    #[error("quote {0} not found")]
    QuoteNotFound(String),
    #[error("unknown node id: {0}")]
    NodeId(u32),
    #[error("failed to get the node url from the db: {0}")]
    GetNodeUrl(rusqlite::Error),
}
