#[derive(Debug, thiserror::Error)]
pub enum GetNodesBalanceError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
}

impl serde::Serialize for GetNodesBalanceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddNodeError {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error("invalid node url: {0}")]
    InvalidNodeUrl(#[from] wallet::types::NodeUrlError),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error), // TODO: create more granular errors in wallet
}

impl serde::Serialize for AddNodeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
