use nuts::{nut01::PublicKey, nut07::ProofCheckState};
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
        ys: Vec<Vec<u8>>,
    ) -> Result<nuts::nut07::CheckStateResponse, Error> {
        let mut conn = self.pg_pool.acquire().await.map_err(Error::DbConnection)?;

        let existing_proofs = db_node::proof::get_proofs_by_ids(&mut conn, ys.clone().into_iter())
            .await
            .map_err(Error::ProofStateRetrieval)?;

        // Handle PublicKey conversion errors properly
        let proof_states: Result<Vec<nuts::nut07::ProofCheckState>, Error> = ys
            .iter()
            .map(|y| {
                let public_key = PublicKey::from_slice(y).map_err(Error::InvalidPublicKey)?;
                let state = existing_proofs
                    .iter()
                    .find(|proof| proof.y.serialize() == *y.as_slice())
                    .map_or(nuts::nut07::ProofState::Unspent, |proof| {
                        proof.state.clone()
                    });

                Ok(ProofCheckState {
                    y: public_key,
                    state,
                })
            })
            .collect();

        let proof_states = proof_states?;

        Ok(nuts::nut07::CheckStateResponse {
            proof_check_states: proof_states,
        })
    }
}
