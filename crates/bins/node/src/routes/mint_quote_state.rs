use nuts::nut04::MintQuoteResponse;
use tonic::Status;
use uuid::Uuid;

use crate::{app_state::AppState, methods::Method};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Db(#[from] db_node::Error),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

impl AppState {
    pub async fn inner_mint_quote_state(
        &self,
        method: Method,
        quote_id: Uuid,
    ) -> Result<Option<MintQuoteResponse<Uuid>>, Error> {
        match method {
            Method::Starknet => {}
        }

        let mut conn = self.pg_pool.acquire().await?;

        let mint_quote_response =
            db_node::mint_quote::build_response_from_db(&mut conn, quote_id).await?;

        Ok(mint_quote_response)
    }
}
