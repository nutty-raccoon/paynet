use nuts::nut01::PublicKey;
use tonic::Status;

use crate::grpc_service::GrpcState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Database errors
    #[error("failed to acquire database connection: {0}")]
    DbConnection(#[source] sqlx::Error),
    #[error("failed to retrieve proof state: {0}")]
    ProofStateRetrieval(#[source] sqlx::Error),
    #[error("invalid public key: {0}")]
    InvalidPublicKey(#[source] nuts::nut01::Error),
}

impl From<Error> for Status {
    fn from(error: Error) -> Self {
        match error {
            Error::DbConnection(_) => Status::internal("Database connection error"),
            Error::ProofStateRetrieval(_) => Status::not_found("Failed to retrieve proof state"),
            Error::InvalidPublicKey(_) => Status::invalid_argument("Invalid public key provided"),
        }
    }
}
impl GrpcState {
    pub async fn inner_check_state(
        &self,
        ys: Vec<PublicKey>,
    ) -> Result<nuts::nut07::CheckStateResponse, Error> {
        let mut conn = self.pg_pool.acquire().await.map_err(Error::DbConnection)?;

        let proof_data = db_node::proof::get_proofs_by_ids(&mut conn, &ys)
            .await
            .unwrap();

        let proof_states: Result<Vec<_>, Error> = proof_data
            .iter()
            .zip(ys.iter())
            .map(|(state, y)| {
                Ok(nuts::nut07::ProofCheckState {
                    y: *y,
                    state: state.clone(),
                })
            })
            .collect();

        Ok(nuts::nut07::CheckStateResponse {
            proof_check_states: proof_states?,
        })
    }
}
