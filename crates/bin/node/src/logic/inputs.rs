use futures::TryFutureExt;
use num_traits::CheckedAdd;
use starknet_types::Unit;
use std::collections::HashSet;
use thiserror::Error;
use tonic::Status;

use db_node::InsertSpentProofsQueryBuilder;
use nuts::{Amount, nut00::Proof, nut01::PublicKey};
use signer::VerifyProofsRequest;
use sqlx::PgConnection;

use crate::{
    app_state::SharedSignerClient,
    keyset_cache::{self, KeysetCache},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to compute y by running hash_on_curve")]
    HashOnCurve,
    #[error("Duplicate input")]
    DuplicateInput,
    #[error("Melt only support inputs of the same unit")]
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
    #[error("Invalid Proof")]
    Invalid,
    #[error("Proof already used")]
    Used,
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
            | Error::Used => Status::invalid_argument(value.to_string()),
            Error::Db(sqlx::Error::RowNotFound) => Status::not_found(value.to_string()),
            Error::Db(_) | Error::KeysetCache(_) => Status::internal(value.to_string()),
            Error::Signer(status) => status,
        }
    }
}

pub async fn process_melt_inputs<'a>(
    conn: &mut PgConnection,
    signer: SharedSignerClient,
    keyset_cache: KeysetCache,
    inputs: &'a [Proof],
) -> Result<(Amount, InsertSpentProofsQueryBuilder<'a>), Error> {
    let mut common_unit = None;
    let mut secrets = HashSet::new();
    let mut query_builder = InsertSpentProofsQueryBuilder::new();
    let mut total_amount = Amount::ZERO;

    let mut verify_proofs_request = Vec::with_capacity(inputs.len());

    for proof in inputs {
        let y = proof.y().map_err(|_| Error::HashOnCurve)?;
        // Uniqueness
        if !secrets.insert(y) {
            Err(Error::DuplicateInput)?;
        }

        let keyset_info = keyset_cache
            .get_keyset_info(conn, proof.keyset_id)
            .await
            .map_err(Error::KeysetCache)?;

        // Check all units are the same
        let unit = keyset_info.1;
        match common_unit {
            Some(u) => {
                if u != unit {
                    Err(Error::MultipleUnits)?;
                }
            }
            None => common_unit = Some(unit),
        }

        // Incement total amount
        total_amount = total_amount
            .checked_add(&proof.amount)
            .ok_or(Error::TotalAmountTooBig)?;

        // Append to insert query
        query_builder.add_row(&y, proof);

        // Prepare payload for verification
        verify_proofs_request.push(signer::Proof {
            amount: proof.amount.into(),
            keyset_id: proof.keyset_id.to_bytes().to_vec(),
            secret: proof.secret.to_string(),
            unblind_signature: proof.c.to_bytes().to_vec(),
        });
    }

    run_verification_queries(conn, secrets, signer, verify_proofs_request).await?;

    Ok((total_amount, query_builder))
}

pub async fn process_swap_inputs<'a>(
    conn: &mut PgConnection,
    signer: SharedSignerClient,
    keyset_cache: KeysetCache,
    inputs: &'a [Proof],
) -> Result<(Vec<(Unit, Amount)>, InsertSpentProofsQueryBuilder<'a>), Error> {
    // Input process
    let mut secrets = HashSet::new();
    let mut amounts_per_unit: Vec<(Unit, Amount)> = Vec::new();
    let mut query_builder = InsertSpentProofsQueryBuilder::new();

    let mut verify_proofs_request = Vec::with_capacity(inputs.len());

    for proof in inputs {
        let y = proof.y().map_err(|_| Error::HashOnCurve)?;
        // Uniqueness
        if !secrets.insert(y) {
            Err(Error::DuplicateInput)?;
        }

        let keyset_info = keyset_cache.get_keyset_info(conn, proof.keyset_id).await?;

        let keyset_unit = keyset_info.1;
        match amounts_per_unit.iter_mut().find(|(u, _)| *u == keyset_unit) {
            Some((_, a)) => {
                *a = a
                    .checked_add(&proof.amount)
                    .ok_or(Error::TotalAmountTooBig)?;
            }
            None => amounts_per_unit.push((keyset_unit, proof.amount)),
        }

        // Append to insert query
        query_builder.add_row(&y, proof);

        // Prepare payload for verification
        verify_proofs_request.push(signer::Proof {
            keyset_id: proof.keyset_id.to_bytes().to_vec(),
            amount: proof.amount.into(),
            secret: proof.secret.to_string(),
            unblind_signature: proof.c.to_bytes().to_vec(),
        });
    }

    run_verification_queries(conn, secrets, signer, verify_proofs_request).await?;

    Ok((amounts_per_unit, query_builder))
}

async fn run_verification_queries(
    conn: &mut PgConnection,
    secrets: HashSet<PublicKey>,
    signer: SharedSignerClient,
    verify_proofs_request: Vec<signer::Proof>,
) -> Result<(), Error> {
    let query_signer_future = async {
        let mut lock = signer.write().await;
        lock.verify_proofs(VerifyProofsRequest {
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
