use futures::TryFutureExt;
use std::collections::HashSet;
use thiserror::Error;
use tonic::Status;

use crate::routes::melt::errors::Error as MeltError;
use nuts::{Amount, nut01::PublicKey, nut02::KeysetId};
use signer::VerifyProofsRequest;
use sqlx::PgConnection;

use crate::{
    app_state::SignerClient,
    keyset_cache::{self},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to compute y by running hash_on_curve")]
    HashOnCurve,
    #[error("duplicate input")]
    DuplicateInput,
    #[error("melt only support inputs of the same unit")]
    MultipleUnits,
    #[error("the sum off all the inputs' amount must fit in a u64")]
    TotalAmountTooBig,
    #[error("the sum off all the inputs' fee must fit in a u64")]
    TotalFeeTooBig,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    KeysetCache(#[from] keyset_cache::Error),
    #[error(transparent)]
    Signer(#[from] tonic::Status),
    #[error("invalid Proof")]
    Invalid,
    #[error("proof already used")]
    Used,
    #[error("amount {1} exceeds max order {2} of keyset {0}")]
    AmountExceedsMaxOrder(KeysetId, Amount, u64),
    #[error("invalid address `{addr}`: {message}")]
    InvalidAddress { addr: String, message: String },
    #[error("other error: {0}")]
    Other(String),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::HashOnCurve
            | Error::DuplicateInput
            | Error::MultipleUnits
            | Error::TotalAmountTooBig
            | Error::TotalFeeTooBig
            | Error::Invalid
            | Error::Used
            | Error::AmountExceedsMaxOrder(_, _, _) => Status::invalid_argument(value.to_string()),
            Error::Db(sqlx::Error::RowNotFound) => Status::not_found(value.to_string()),
            Error::Db(_) | Error::KeysetCache(_) => Status::internal(value.to_string()),
            Error::Signer(status) => status,
            Error::InvalidAddress { addr, message } => {
                Status::invalid_argument(format!("Invalid address `{addr}`: {message}"))
            }
            Error::Other(message) => Status::unknown(message),
        }
    }
}

impl From<MeltError> for Error {
    fn from(error: MeltError) -> Self {
        match error {
            MeltError::InvalidAddress { addr, message } => Self::InvalidAddress { addr, message },
            _ => Self::Other(format!("{:?}", error)),
        }
    }
}

pub async fn run_verification_queries(
    conn: &mut PgConnection,
    secrets: HashSet<PublicKey>,
    mut signer: SignerClient,
    verify_proofs_request: Vec<signer::Proof>,
) -> Result<(), Error> {
    let query_signer_future = async {
        signer
            .verify_proofs(VerifyProofsRequest {
                proofs: verify_proofs_request,
            })
            .await
    };

    // Parrallelize the two calls
    let res = tokio::try_join!(
        // Make sure those proof are valid
        query_signer_future
            .map_ok(|r| r.get_ref().is_valid)
            .map_err(Error::Signer),
        // Make sure those inputs were not already used
        db_node::is_any_proof_already_used(conn, secrets.into_iter()).map_err(Error::Db),
    );

    match res {
        Ok((false, _)) => Err(Error::Invalid),
        Ok((_, true)) => Err(Error::Used),
        Err(e) => Err(e),
        Ok((true, false)) => Ok(()),
    }
}
