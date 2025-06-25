use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut02::KeysetId,
};
use tonic::{Code, Status};

use crate::grpc_service::GrpcState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to acquire connection from the pool: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to get the secrets from database: {0}")]
    Db(#[from] db_node::Error),
    #[error("could not find blind signature for those blind messages: {0:?}")]
    MissingBlindMessage(Vec<BlindedMessage>),
    #[error("could not find blind signature for those blind messages: {0:?}")]
    BlindMessageMismatch(Vec<(BlindedMessage, (KeysetId, Amount))>),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::Sqlx(error) => Status::internal(error.to_string()),
            Error::Db(error) => Status::internal(error.to_string()),
            Error::MissingBlindMessage(blinded_messages) => {
                let error_details = match serde_json::to_vec(&blinded_messages).map_err(|e| {
                    Status::internal(format!(
                        "failed to serialize list of not found blind messages: {e}"
                    ))
                }) {
                    Ok(bytes) => bytes,
                    Err(e) => return e,
                };
                Status::with_details(
                    Code::NotFound,
                    "missing matching signatures for blind messages",
                    error_details.into(),
                )
            }
            Error::BlindMessageMismatch(non_matching_blind_messages) => {
                let error_details =
                    match serde_json::to_vec(&non_matching_blind_messages).map_err(|e| {
                        Status::internal(format!(
                            "failed to serialize list of non-matching blind messages: {e}"
                        ))
                    }) {
                        Ok(bytes) => bytes,
                        Err(e) => return e,
                    };
                Status::with_details(
                    Code::InvalidArgument,
                    "those blind messages amount and keyset id don't match the ones for their blind secret",
                    error_details.into(),
                )
            }
        }
    }
}

impl GrpcState {
    pub async fn inner_restore(
        &self,
        blind_messages: Vec<BlindedMessage>,
    ) -> Result<Vec<BlindSignature>, Error> {
        let mut conn = self.pg_pool.acquire().await?;
        let signatures = db_node::blind_signature::get_by_blind_secrets(
            &mut conn,
            blind_messages.iter().map(|bm| bm.blinded_secret),
        )
        .await?;

        if signatures.len() != blind_messages.len() {
            let mut not_found_blind_messages = Vec::new();
            for blind_message in blind_messages {
                if !signatures.iter().any(|bs| {
                    bs.amount == blind_message.amount && bs.keyset_id == blind_message.keyset_id
                }) {
                    not_found_blind_messages.push(blind_message);
                }
            }
            return Err(Error::MissingBlindMessage(not_found_blind_messages));
        } else {
            let mut non_matching_blind_messages = Vec::new();
            for (bs, bm) in signatures.iter().zip(blind_messages.into_iter()) {
                if bs.amount != bm.amount || bs.keyset_id != bm.keyset_id {
                    non_matching_blind_messages.push((bm, (bs.keyset_id, bs.amount)))
                }
            }

            if !non_matching_blind_messages.is_empty() {
                return Err(Error::BlindMessageMismatch(non_matching_blind_messages));
            }
        };

        Ok(signatures)
    }
}
