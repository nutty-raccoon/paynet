use node_client::UnspecifiedEnum;
use nuts::nut01::PublicKey;
use rusqlite::Connection;
use thiserror::Error;
use tonic::Status;
use tonic_types::StatusExt;

use crate::{StoreNewProofsError, db, seed_phrase};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("unknown enum value: {0}")]
    ProstUnknownEnumValue(#[from] prost::UnknownEnumValue),
    #[error(transparent)]
    UnspecifiedEnum(#[from] UnspecifiedEnum),
    #[error("amount overflow")]
    AmountOverflow,
    #[error("no matching keyset found")]
    NoMatchingKeyset,
    #[error("proof not available")]
    ProofNotAvailable,
    #[error("invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("invalid unit: {0}")]
    InvalidUnit(String),
    #[error("invalid keyset ID")]
    InvalidKeysetId(#[from] std::array::TryFromSliceError),
    #[error("gRPC error: {0}")]
    Grpc(#[from] Status),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("not enough funds")]
    NotEnoughFunds,
    #[error("nut01 error: {0}")]
    Nut01(#[from] nuts::nut01::Error),
    #[error("nut02 error: {0}")]
    Nut02(#[from] nuts::nut02::Error),
    #[error("nut13 error: {0}")]
    Nut13(#[from] nuts::nut13::Error),
    #[error("bdhke error: {0}")]
    Dhke(#[from] nuts::dhke::Error),
    #[error("conversion error: {0}")]
    Conversion(String),
    #[error("nuts error: {0}")]
    Nuts(#[from] nuts::Error),
    #[error("Secret error: {0}")]
    Secret(#[from] nuts::nut00::secret::Error),
    #[error("keyset unit mismatch, expected {0} got {0}")]
    UnitMissmatch(String, String),
    #[error("failed to get a connection from the pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
    #[error(transparent)]
    Wallet(#[from] crate::wallet::Error),
    #[error(transparent)]
    RestoreNode(#[from] crate::node::RestoreNodeError),
    #[error("unexpected proof state: {0}")]
    UnexpectedProofState(String),
    #[error("failed to connect to node: {0}")]
    ConnectToNode(#[from] crate::ConnectToNodeError),
    #[error("invalid field format: '[' or ']' not found")]
    InvalidFormat,
    #[error("invalid index: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

impl From<StoreNewProofsError> for Error {
    fn from(value: StoreNewProofsError) -> Self {
        match value {
            StoreNewProofsError::Rusqlite(error) => Error::Database(error),
            StoreNewProofsError::Nut01(error) => Error::Nut01(error),
            StoreNewProofsError::Dhke(error) => Error::Dhke(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn handle_proof_verification_errors(
    status: &Status,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<()> {
    let error_details = status.get_error_details();

    if let Some(bad_request) = error_details.bad_request() {
        let mut crypto_failed_indices = Vec::new();
        let mut already_spent_indices = Vec::new();

        for violation in &bad_request.field_violations {
            let proof_index = extract_proof_index(&violation.field)?;

            match &violation.description {
                desc if desc.contains("failed cryptographic verification") => {
                    crypto_failed_indices.push(proof_index);
                }
                desc if desc.contains("already spent") => {
                    already_spent_indices.push(proof_index);
                }
                _ => {
                    log::error!(
                        "Unknown proof error for index {}: {}",
                        proof_index,
                        violation.description
                    );
                }
            }
        }

        if !crypto_failed_indices.is_empty() {
            handle_crypto_invalid_proofs(crypto_failed_indices, proofs_ids, conn)?;
        }

        if !already_spent_indices.is_empty() {
            handle_already_spent_proofs(already_spent_indices, proofs_ids, conn)?;
        }
    }
    Ok(())
}

fn handle_crypto_invalid_proofs(
    indices: Vec<u32>,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<()> {
    log::info!(
        "Removing {} cryptographically invalid proofs: {:?}",
        indices.len(),
        indices
    );

    remove_invalid_proofs(indices, proofs_ids, conn)?;
    Ok(())
}

fn handle_already_spent_proofs(
    indices: Vec<u32>,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<()> {
    log::info!(
        "Removing {} already spent proofs: {:?}",
        indices.len(),
        indices
    );

    remove_invalid_proofs(indices, proofs_ids, conn)?;
    Ok(())
}

fn remove_invalid_proofs(
    indices: Vec<u32>,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<()> {
    log::info!("Removing {} invalid proofs: {:?}", indices.len(), indices);

    let invalid_proofs = proofs_ids
        .iter()
        .enumerate()
        .filter_map(|(i, id)| {
            if indices.contains(&(i as u32)) {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<Vec<PublicKey>>();

    db::proof::delete_proofs(conn, &invalid_proofs)?;
    Ok(())
}

fn extract_proof_index(field: &str) -> Result<u32> {
    if let Some(start) = field.find('[') {
        if let Some(end) = field.find(']') {
            let index_str = &field[start + 1..end];
            return Ok(index_str.parse::<u32>()?);
        }
    }
    Err(Error::InvalidFormat)
}
