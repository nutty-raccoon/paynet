use num_traits::CheckedAdd;
use std::collections::HashSet;

use db_node::InsertSpentProofsQueryBuilder;
use nuts::{Amount, nut00::Proof};
use sqlx::PgConnection;
use starknet_types::Unit;

use crate::{
    app_state::SignerClient,
    keyset_cache::KeysetCache,
    logic::{InputsError, run_inputs_verification_queries},
};

pub async fn process_swap_inputs<'a>(
    conn: &mut PgConnection,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    inputs: &'a [Proof],
) -> Result<(Vec<(Unit, Amount)>, InsertSpentProofsQueryBuilder<'a>), InputsError> {
    // Input process
    let mut secrets = HashSet::new();
    let mut amounts_per_unit: Vec<(Unit, Amount)> = Vec::new();
    let mut query_builder = InsertSpentProofsQueryBuilder::new();

    let mut verify_proofs_request = Vec::with_capacity(inputs.len());

    for proof in inputs {
        let y = proof.y().map_err(|_| InputsError::HashOnCurve)?;
        // Uniqueness
        if !secrets.insert(y) {
            Err(InputsError::DuplicateInput)?;
        }

        let keyset_info = keyset_cache.get_keyset_info(conn, proof.keyset_id).await?;

        let keyset_unit = keyset_info.unit();

        // Validate amount doesn't exceed max_order
        let max_order = keyset_info.max_order();
        let max_value = 1u64 << (max_order - 1);

        if u64::from(proof.amount) > max_value {
            return Err(InputsError::AmountExceedsMaxOrder(
                proof.keyset_id,
                proof.amount,
                max_value,
            ));
        }

        match amounts_per_unit.iter_mut().find(|(u, _)| *u == keyset_unit) {
            Some((_, a)) => {
                *a = a
                    .checked_add(&proof.amount)
                    .ok_or(InputsError::TotalAmountTooBig)?;
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

    run_inputs_verification_queries(conn, secrets, signer, verify_proofs_request).await?;

    Ok((amounts_per_unit, query_builder))
}
