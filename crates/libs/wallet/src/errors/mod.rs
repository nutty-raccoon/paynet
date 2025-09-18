use cashu_client::CashuClient;
use node_client::{NodeClient, UnspecifiedEnum};
use nuts::nut01::PublicKey;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use thiserror::Error;
use tonic::{Code, Status, transport::Channel};
use tonic_types::StatusExt;
use tracing::{error, info};

use crate::{StoreNewProofsError, db, node::RefreshNodeKeysetError, seed_phrase};

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
    #[error("fail to refresh node keyset: {0}")]
    RefreshNodeKeyset(#[from] RefreshNodeKeysetError),
    #[error(transparent)]
    Cashu(#[from] cashu_client::Error),
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

pub fn handle_crypto_invalid_proofs(
    indices: Vec<u32>,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<(), rusqlite::Error> {
    info!(
        "Removing {} cryptographically invalid proofs: {:?}",
        indices.len(),
        indices
    );

    let mut invalid_proofs: Vec<PublicKey> = vec![];
    for i in &indices {
        if let Some(id) = proofs_ids.get(*i as usize) {
            invalid_proofs.push(*id);
        } else {
            error!("Invalid index: {}", i);
        }
    }

    db::proof::delete_proofs(conn, &invalid_proofs)?;
    Ok(())
}

pub fn handle_already_spent_proofs(
    indices: Vec<u32>,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<(), rusqlite::Error> {
    info!(
        "Removing {} already spent proofs: {:?}",
        indices.len(),
        indices
    );

    let mut invalid_proofs: Vec<PublicKey> = vec![];
    for i in &indices {
        if let Some(id) = proofs_ids.get(*i as usize) {
            invalid_proofs.push(*id);
        } else {
            error!("Node returned an out of bound index for invalid proof: {i}",);
        }
    }

    db::proof::set_proofs_to_state(conn, &invalid_proofs, crate::types::ProofState::Spent)?;
    Ok(())
}

fn extract_proof_index(field: &str) -> Result<u32, Error> {
    if let Some(start) = field.find('[') {
        if let Some(end) = field.find(']') {
            let index_str = &field[start + 1..end];
            return Ok(index_str.parse::<u32>()?);
        }
    }

    Err(Error::InvalidFormat)
}

#[cfg(test)]
mod tests {
    use super::extract_proof_index;

    #[test]
    fn test_extract_proof_index_valid_input() {
        let cases = [
            ("proofs[5]", 5),
            ("proofs[10]", 10),
            ("proofs[123]", 123),
            ("proofs[99]", 99),
        ];

        cases.iter().for_each(|(field, expected)| {
            assert_eq!(extract_proof_index(field).unwrap(), *expected)
        });
    }

    #[test]
    fn test_extract_proof_index_invalid_format() {
        let cases = ["proofs5]", "proofs[5", "", "proofs74]", "proofs[980"];

        cases.iter().for_each(|field| {
            assert!(extract_proof_index(field).is_err());
            assert_eq!(
                extract_proof_index(field).unwrap_err().to_string(),
                "invalid field format: '[' or ']' not found"
            )
        });
    }

    #[test]
    fn test_extract_proof_index_parse_error() {
        let cases = ["proofs[abc]", "proofs[1.2]", "proofs[9a]", "proofs[a4]"];

        cases.iter().for_each(|field| {
            assert!(extract_proof_index(field).is_err());
            assert_eq!(
                extract_proof_index(field).unwrap_err().to_string(),
                "invalid index: invalid digit found in string"
            );
        });
    }

    #[test]
    fn test_extract_proof_index_edge_cases() {
        let cases: [(&str, u32); 2] = [("proofs[0]", 0), ("proofs[4294967295]", 4294967295)];

        cases.iter().for_each(|(field, element)| {
            assert_eq!(extract_proof_index(field).unwrap(), *element);
        });

        assert!(extract_proof_index("proofs[4294967296]").is_err());
        assert!(extract_proof_index("proofs[-1]").is_err());
    }
}
