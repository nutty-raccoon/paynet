use crate::grpc_service::GrpcState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Database errors
    #[error("failed to acquire database connection: {0}")]
    DbConnection(#[source] sqlx::Error),
    #[error("failed to retrieve proof state: {0}")]
    ProofStateRetrieval(#[source] sqlx::Error),
    // Other errors
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
impl GrpcState {
    pub async fn inner_check_state(
        &self,
        ys: Vec<String>,
    ) -> Result<nuts::nut07::PostCheckStateResponse, Error> {
        let mut conn = self.pg_pool.acquire().await.map_err(Error::DbConnection)?;

        let proof_check_states =
            db_node::check_proof_state::get_proofs_by_y(&mut conn, ys.into_iter())
                .await
                .map_err(Error::ProofStateRetrieval)?;

        Ok(nuts::nut07::PostCheckStateResponse { proof_check_states })
    }
}
