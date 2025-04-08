#[derive(Debug, thiserror::Error)]
pub enum GetNodesBalanceError {
    #[error(transparent)]
    Rusqlite(rusqlite::Error),
    #[error("the tauri state mutex has been poisoned")]
    StateMutexPoisoned,
}

impl serde::Serialize for GetNodesBalanceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
