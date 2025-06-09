use nuts::{nut01, nut02};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Init(#[from] crate::initialization::Error),
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
    #[error(transparent)]
    Nut02(#[from] nut02::Error),
    #[error(transparent)]
    Database(#[from] db_node::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Tonic(#[from] tonic::transport::Error),
    #[error(transparent)]
    Signer(#[from] tonic::Status),
}
