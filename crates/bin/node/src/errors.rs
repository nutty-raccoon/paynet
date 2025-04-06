use axum::{Json, http::StatusCode, response::IntoResponse};
use nuts::{dhke, nut00::CashuError, nut01, nut02};
use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Init(#[from] crate::initialization::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Dhke(#[from] dhke::Error),
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
    #[error(transparent)]
    Nut02(#[from] nut02::Error),
    #[error(transparent)]
    Database(#[from] db_node::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Tonic(#[from] tonic::Status),
    #[error(transparent)]
    Service(#[from] ServiceError),
    #[error("Keyset doesn't exist in this mint")]
    UnknownKeySet,
    #[error("No keypair for amount")]
    InvalidAmountKey,
    #[error("A value overflowed during execution")]
    Overflow,
    #[error("The KeyManager generated a KeysetId different from the one known in db")]
    GeneratedKeysetIdIsDifferentFromOriginal,
    /// Inactive Keyset
    #[error("Inactive Keyset")]
    InactiveKeyset,
}

impl From<Error> for CashuError {
    fn from(_value: Error) -> Self {
        Self::new(0, String::new())
    }
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(CashuError::from(self))).into_response()
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Failed to run the indexer: {0}")]
    Indexer(#[source] anyhow::Error),
    #[error("Failed to serve the axum server: {0}")]
    AxumServe(#[source] std::io::Error),
    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
}
