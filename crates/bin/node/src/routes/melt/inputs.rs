use num_traits::CheckedAdd;
use starknet_types::Unit;
use std::collections::HashSet;

use db_node::InsertSpentProofsQueryBuilder;
use nuts::{Amount, nut00::Proof};
use sqlx::PgConnection;

use crate::{
    app_state::SignerClient,
    keyset_cache::KeysetCache,
    logic::{InputsError, run_inputs_verification_queries},
};

pub async fn process_melt_inputs<'a>(
    conn: &mut PgConnection,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    inputs: &'a [Proof],
    expected_unit: Unit,
) -> Result<(Amount, InsertSpentProofsQueryBuilder<'a>), InputsError> {
    let mut secrets = HashSet::new();
    let mut query_builder = InsertSpentProofsQueryBuilder::new();
    let mut total_amount = Amount::ZERO;

    let mut verify_proofs_request = Vec::with_capacity(inputs.len());

    for proof in inputs {
        let y = proof.y().map_err(|_| InputsError::HashOnCurve)?;
        // Uniqueness
        if !secrets.insert(y) {
            Err(InputsError::DuplicateInput)?;
        }

        let keyset_info = keyset_cache
            .get_keyset_info(conn, proof.keyset_id)
            .await
            .map_err(InputsError::KeysetCache)?;

        // Validate amount doesn't exceed max_order
        let max_order = keyset_info.max_order();
        let max_value = (1u64 << max_order) - 1;

        if u64::from(proof.amount) > max_value {
            return Err(InputsError::AmountExceedsMaxOrder(
                proof.keyset_id,
                proof.amount,
                max_value,
            ));
        }

        let unit = keyset_info.unit();
        if expected_unit != unit {
            Err(InputsError::UnexpectedUnit)?;
        }

        // Incement total amount
        total_amount = total_amount
            .checked_add(&proof.amount)
            .ok_or(InputsError::TotalAmountTooBig)?;

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

    run_inputs_verification_queries(conn, secrets, signer, verify_proofs_request).await?;

    Ok((total_amount, query_builder))
}
