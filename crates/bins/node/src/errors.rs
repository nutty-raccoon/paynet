use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "keyset-rotation")]
    #[error(transparent)]
    Nut01(#[from] nuts::nut01::Error),
    #[error(transparent)]
    Tonic(tonic::transport::Error),
}
