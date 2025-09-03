#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("failed to get a connection from the pool: {0}")]
    GetConnection(r2d2::Error),
    #[error("failed to interact with db: {0}")]
    Db(rusqlite::Error),
    #[error("failed to create node client: {0}")]
    CreateNodeClient(wallet::ConnectToNodeError),
    #[error(transparent)]
    EmitEvent(tauri::Error),
    #[error("failed to push event in a the mint error channel")]
    MintQuoteChannel,
    #[error("failed wallet logic: {0}")]
    Wallet(#[from] wallet::errors::Error),
    #[error("tauri emit event for front: {0}")]
    TauriEvent(tauri::Error),
    #[error("quote {0} not found")]
    QuoteNotFound(String),
    #[error("unknown node id: {0}")]
    NodeId(u32),
}
