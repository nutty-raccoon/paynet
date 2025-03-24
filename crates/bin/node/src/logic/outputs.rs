use std::collections::HashSet;

use db_node::InsertBlindSignaturesQueryBuilder;
use num_traits::CheckedAdd;
use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut01::PublicKey,
    nut02::KeysetId,
};
use signer::SignBlindedMessagesRequest;
use sqlx::PgConnection;
use starknet_types::Unit;
use thiserror::Error;

use crate::{
    app_state::SignerClient,
    keyset_cache::{self, KeysetCache},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("outputs contains a duplicated element")]
    DuplicateOutput,
    #[error("keyset with id {0} is inactive")]
    InactiveKeyset(KeysetId),
    #[error("this quote require the use of multiple units")]
    MultipleUnits,
    #[error("the sum off all the outputs' amount must fit in a u64")]
    TotalAmountTooBig,
    #[error("blind message is already signed")]
    AlreadySigned,
    #[error("amount exceeds max order for keyset {0}: amount {1} exceeds max value {2}")]
    AmountExceedsMaxOrder(KeysetId, Amount, u64),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Signer(#[from] tonic::Status),
    #[error(transparent)]
    KeysetCache(#[from] keyset_cache::Error),
}

pub async fn check_outputs_allow_single_unit(
    conn: &mut PgConnection,
    keyset_cache: &KeysetCache,
    outputs: &[BlindedMessage],
) -> Result<Amount, Error> {
    let mut blind_secrets = HashSet::with_capacity(outputs.len());
    let mut total_amount = Amount::ZERO;
    let mut unit = None;

    for blind_message in outputs {
        // Uniqueness
        if !blind_secrets.insert(blind_message.blinded_secret) {
            Err(Error::DuplicateOutput)?;
        }

        let keyset_info = keyset_cache
            .get_keyset_info(conn, blind_message.keyset_id)
            .await?;

        // We only sign with active keysets
        if !keyset_info.0 {
            return Err(Error::InactiveKeyset(blind_message.keyset_id));
        }

        match (unit, keyset_info.1) {
            (None, u) => unit = Some(u),
            (Some(unit), u) if u != unit => return Err(Error::MultipleUnits),
            _ => {}
        }

        // Incement total amount
        total_amount = total_amount
            .checked_add(&blind_message.amount)
            .ok_or(Error::TotalAmountTooBig)?;
    }

    // Make sure those outputs were not already signed
    if db_node::is_any_blind_message_already_used(conn, blind_secrets.into_iter()).await? {
        return Err(Error::AlreadySigned);
    }

    Ok(total_amount)
}

pub async fn check_outputs_allow_multiple_units(
    conn: &mut PgConnection,
    keyset_cache: KeysetCache,
    outputs: &[BlindedMessage],
) -> Result<Vec<(Unit, Amount)>, Error> {
    let mut blind_secrets = HashSet::with_capacity(outputs.len());
    let mut total_amounts: Vec<(Unit, Amount)> = Vec::new();

    for blind_message in outputs {
        // Uniqueness
        if !blind_secrets.insert(blind_message.blinded_secret) {
            return Err(Error::DuplicateOutput);
        }

        let keyset_info = keyset_cache
            .get_keyset_info(conn, blind_message.keyset_id)
            .await?;

        // We only sign with active keysets
        if !keyset_info.0 {
            return Err(Error::InactiveKeyset(blind_message.keyset_id));
        }

        // Validate amount doesn't exceed max_order
        let max_order = keyset_info.2;
        let max_value = if max_order >= 64 {
            u64::MAX
        } else {
            (1u64 << max_order) - 1
        };

        if u64::from(blind_message.amount) > max_value {
            return Err(Error::AmountExceedsMaxOrder(
                blind_message.keyset_id,
                blind_message.amount,
                max_value,
            ));
        }

        // Incement total amount
        let keyset_unit = keyset_info.1;
        match total_amounts.iter_mut().find(|(u, _)| *u == keyset_unit) {
            Some((_, a)) => {
                *a = a
                    .checked_add(&blind_message.amount)
                    .ok_or(Error::TotalAmountTooBig)?
            }
            None => total_amounts.push((keyset_unit, blind_message.amount)),
        }
    }

    // Make sure those outputs were not already signed
    if db_node::is_any_blind_message_already_used(conn, blind_secrets.into_iter()).await? {
        Err(Error::AlreadySigned)?;
    }

    Ok(total_amounts)
}

pub async fn process_outputs<'a>(
    mut signer: SignerClient,
    outputs: &[BlindedMessage],
) -> Result<(Vec<BlindSignature>, InsertBlindSignaturesQueryBuilder<'a>), Error> {
    let mut query_builder = InsertBlindSignaturesQueryBuilder::new();

    let blind_signatures = signer
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: outputs
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await?
        .into_inner()
        .signatures;

    let blind_signatures = outputs
        .iter()
        .zip(blind_signatures)
        .map(|(bm, bs)| {
            let blind_signature = BlindSignature {
                amount: bm.amount,
                keyset_id: bm.keyset_id,
                c: PublicKey::from_slice(&bs).expect("the signer should return valid pubkey"),
            };

            query_builder.add_row(bm.blinded_secret, &blind_signature);

            blind_signature
        })
        .collect::<Vec<_>>();

    Ok((blind_signatures, query_builder))
}
