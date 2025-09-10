use std::collections::HashSet;

use node_client::{NodeClient, UnspecifiedEnum};
use nuts::{Amount, nut01::PublicKey, nut02::KeysetId};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use thiserror::Error;
use tonic::{Code, Status, transport::Channel};
use tonic_types::StatusExt;
use tracing::{error, info};

use crate::{
    StoreNewProofsError, db,
    node::RefreshNodeKeysetError,
    seed_phrase,
    sync::{SyncMeltQuoteError, SyncMeltQuotesError},
};

#[derive(Debug, thiserror::Error)]
pub enum CommonError {
    #[error("proof amounts must be powers of two, got {0}")]
    ProofAmountPowerOfTwo(u64),
    #[error("proof amounts cannot be greater that the keyset {0} max order {1}, got {2}")]
    ProofAmountGreaterThanKeysetMaxOrder(KeysetId, u64, u64),
    #[error("failed to hash the secret to the curve: {0}")]
    HashToCurve(#[source] nuts::dhke::Error),
    #[error("failed to start a database transaction: {0}")]
    CreateDbTransaction(#[source] rusqlite::Error),
    #[error("failed to commit a database transaction: {0}")]
    CommitDbTransaction(#[source] rusqlite::Error),
    #[error("failed to get database connection from the pool: {0}")]
    GetDbConnection(#[source] r2d2::Error),
    #[error("failed to load blinding data: {0}")]
    LoadBlindingData(#[source] crate::Error),
    #[error("failed to generate Premints for amount {0}: {1}")]
    GeneratePremintsForAmount(Amount, #[source] crate::Error),
    #[error("failed to handle the proofs verification errors: {0}")]
    HandleProofVerificationErrors(#[source] crate::Error),
    #[error("failed to store the PreMints new tokens: {0}")]
    PreMintsStoreNewTokens(#[source] crate::Error),
    #[error("failed to acknowledge node response to {0}: {0}")]
    AcknowledgeNodeResponse(&'static str, #[source] tonic::Status),
}

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
    #[error("failed to sync melt quotes: {0}")]
    SyncMeltQuotes(#[from] SyncMeltQuotesError),
    #[error("failed to sync melt quote: {0}")]
    SyncMeltQuote(#[from] SyncMeltQuoteError),
    #[error(transparent)]
    SetProofsToState(#[from] db::proof::SetProofsToStateError),
    #[error(transparent)]
    GetProofsByIds(#[from] db::proof::GetProofsByIdsError),
    #[error(transparent)]
    UpdateWadStatusError(#[from] db::wad::UpdateWadStatusError),
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

pub async fn handle_out_of_sync_keyset_errors(
    status: &Status,
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
) -> Result<(), RefreshNodeKeysetError> {
    let mut should_refresh = false;
    if status.code() == Code::FailedPrecondition && status.message() == "inactive keyset" {
        let error_details = status.get_error_details();
        if let Some(precondition_failure) = error_details.precondition_failure() {
            for failure in &precondition_failure.violations {
                if failure.r#type == "keyset.state" {
                    should_refresh = true;
                }
            }
        }
    }

    if should_refresh {
        crate::node::refresh_keysets(pool, node_client, node_id).await?;
    }

    Ok(())
}

pub fn handle_proof_verification_errors(
    status: &Status,
    proofs_ids: &[PublicKey],
    conn: &Connection,
) -> Result<(), Error> {
    let error_details = status.get_error_details();

    if let Some(bad_request) = error_details.bad_request() {
        let mut crypto_failed_indices = HashSet::new();
        let mut already_spent_indices = HashSet::new();

        for violation in &bad_request.field_violations {
            let proof_index = extract_proof_index(&violation.field)?;

            match &violation.description {
                desc if desc.contains("failed cryptographic verification") => {
                    crypto_failed_indices.insert(proof_index as usize);
                }
                desc if desc.contains("already spent") => {
                    already_spent_indices.insert(proof_index as usize);
                }
                _ => {
                    error!(
                        "Unknown proof error for index {}: {}",
                        proof_index, violation.description
                    );
                }
            }
        }

        let mut invalid_crypto_ids = Vec::new();
        let mut already_spent_ids = Vec::new();
        let mut other_ids = Vec::new();

        for (i, item) in proofs_ids.iter().enumerate() {
            if crypto_failed_indices.contains(&i) {
                invalid_crypto_ids.push(*item);
            } else if already_spent_indices.contains(&i) {
                already_spent_ids.push(*item);
            } else {
                other_ids.push(*item);
            }
        }

        if !invalid_crypto_ids.is_empty() {
            info!(
                "Removing {} cryptographically invalid proofs: {:?}",
                proofs_ids.len(),
                proofs_ids
            );

            db::proof::delete_proofs(conn, &invalid_crypto_ids)?;
        }
        if !already_spent_ids.is_empty() {
            info!(
                "Removing {} already spent proofs: {:?}",
                proofs_ids.len(),
                proofs_ids
            );

            db::proof::set_proofs_to_state(
                conn,
                &already_spent_ids,
                crate::types::ProofState::Spent,
            )?;
        }
        if !other_ids.is_empty() {
            db::proof::set_proofs_to_state(conn, &other_ids, crate::types::ProofState::Unspent)?;
        }
    }
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
