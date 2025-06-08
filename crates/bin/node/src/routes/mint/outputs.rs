use num_traits::CheckedAdd;
use std::collections::HashSet;

use nuts::{Amount, nut00::BlindedMessage};
use sqlx::PgConnection;

use crate::{keyset_cache::KeysetCache, logic::OutputsError};

pub async fn check_outputs_allow_single_unit(
    conn: &mut PgConnection,
    keyset_cache: &KeysetCache,
    outputs: &[BlindedMessage],
) -> Result<Amount, OutputsError> {
    let mut blind_secrets = HashSet::with_capacity(outputs.len());
    let mut total_amount = Amount::ZERO;
    let mut unit = None;

    for blind_message in outputs {
        // Uniqueness
        if !blind_secrets.insert(blind_message.blinded_secret) {
            Err(OutputsError::DuplicateOutput)?;
        }

        let keyset_info = keyset_cache
            .get_keyset_info(conn, blind_message.keyset_id)
            .await?;

        // We only sign with active keysets
        if !keyset_info.active() {
            return Err(OutputsError::InactiveKeyset(blind_message.keyset_id));
        }

        match (unit, keyset_info.unit()) {
            (None, u) => unit = Some(u),
            (Some(unit), u) if u != unit => return Err(OutputsError::MultipleUnits),
            _ => {}
        }

        // Incement total amount
        total_amount = total_amount
            .checked_add(&blind_message.amount)
            .ok_or(OutputsError::TotalAmountTooBig)?;
    }

    // Make sure those outputs were not already signed
    if db_node::is_any_blind_message_already_used(conn, blind_secrets.into_iter()).await? {
        return Err(OutputsError::AlreadySigned);
    }

    Ok(total_amount)
}
