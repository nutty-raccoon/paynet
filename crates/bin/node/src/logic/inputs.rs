use futures::TryFutureExt;
use std::collections::HashSet;
use thiserror::Error;
use tonic::Status;

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
    #[error("the proofs were not of the quote unit")]
    UnexpectedUnit,
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
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::HashOnCurve
            | Error::DuplicateInput
            | Error::UnexpectedUnit
            | Error::TotalAmountTooBig
            | Error::TotalFeeTooBig
            | Error::Invalid
            | Error::Used
            | Error::AmountExceedsMaxOrder(_, _, _) => Status::invalid_argument(value.to_string()),
            Error::Db(sqlx::Error::RowNotFound) => Status::not_found(value.to_string()),
            Error::Db(_) | Error::KeysetCache(_) => Status::internal(value.to_string()),
            Error::Signer(status) => status,
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

    // Parallelize the two calls
    let res = tokio::try_join!(
        // Make sure those proof are valid
        query_signer_future
            .map_ok(|r| r.get_ref().is_valid)
            .map_err(Error::Signer),
        // Make sure those inputs were not already used
        db_node::proof::is_any_already_spent(conn, secrets.into_iter()).map_err(Error::Db),
    );

    match res {
        Ok((false, _)) => Err(Error::Invalid),
        Ok((_, true)) => Err(Error::Used),
        Err(e) => Err(e),
        Ok((true, false)) => Ok(()),
    }
}
