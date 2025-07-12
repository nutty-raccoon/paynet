use db_node::blind_signature::RestoreFromDbResponse;
use nuts::nut00::BlindedMessage;
use tonic::Status;

use crate::grpc_service::GrpcState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to acquire connection from the pool: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to get the secrets from database: {0}")]
    Db(#[from] db_node::Error),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::Sqlx(error) => Status::internal(error.to_string()),
            Error::Db(error) => Status::internal(error.to_string()),
        }
    }
}

impl GrpcState {
    pub async fn inner_restore(
        &self,
        blind_messages: Vec<BlindedMessage>,
    ) -> Result<Vec<RestoreFromDbResponse>, Error> {
        let mut conn = self.pg_pool.acquire().await?;
        let secret_with_signatures = db_node::blind_signature::get_by_blind_secrets(
            &mut conn,
            blind_messages.iter().map(|bm| bm.blinded_secret),
        )
        .await?;

        Ok(secret_with_signatures)
    }
}
