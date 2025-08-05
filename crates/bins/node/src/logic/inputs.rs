use std::collections::HashSet;
use thiserror::Error;
use tonic::{Code, Status};
use tonic_types::{ErrorDetails, FieldViolation, StatusExt};

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
    #[error("amount {1} exceeds max order {2} of keyset {0}")]
    AmountExceedsMaxOrder(KeysetId, Amount, u64),
    #[error("proof issues found")]
    ProofIssues {
        invalid_crypto_indices: Vec<u32>,
        spent_proof_indices: Vec<u32>,
    },
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::HashOnCurve
            | Error::DuplicateInput
            | Error::UnexpectedUnit
            | Error::TotalAmountTooBig
            | Error::TotalFeeTooBig
            | Error::AmountExceedsMaxOrder(_, _, _) => Status::invalid_argument(value.to_string()),
            Error::Db(sqlx::Error::RowNotFound) => Status::not_found(value.to_string()),
            Error::Db(_) | Error::KeysetCache(_) => Status::internal(value.to_string()),
            Error::Signer(status) => status,
            Error::ProofIssues {
                invalid_crypto_indices,
                spent_proof_indices,
            } => Status::with_error_details(
                Code::InvalidArgument,
                "proof verification failed",
                ErrorDetails::with_bad_request(
                    invalid_crypto_indices
                        .iter()
                        .map(|&idx| {
                            FieldViolation::new(
                                format!("proofs[{}]", idx),
                                "proof failed cryptographic verification".to_string(),
                            )
                        })
                        .chain(spent_proof_indices.iter().map(|&idx| {
                            FieldViolation::new(
                                format!("proofs[{}]", idx),
                                "proof already spent".to_string(),
                            )
                        }))
                        .collect::<Vec<FieldViolation>>(),
                ),
            ),
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
            .map_err(Error::Signer)
    };
    let spent_check_future = async {
        db_node::proof::get_already_spent_indices(conn, secrets.into_iter())
            .await
            .map_err(Error::Db)
    };

    // Parallelize the two calls
    match tokio::try_join!(query_signer_future, spent_check_future) {
        Ok((signer_response, spent_proof_indices)) => {
            let invalid_crypto_indices = signer_response.get_ref().invalid_proof_indices.clone();

            if invalid_crypto_indices.is_empty() && spent_proof_indices.is_empty() {
                Ok(())
            } else {
                Err(Error::ProofIssues {
                    invalid_crypto_indices,
                    spent_proof_indices,
                })
            }
        }
        Err(err) => Err(err),
    }
}
