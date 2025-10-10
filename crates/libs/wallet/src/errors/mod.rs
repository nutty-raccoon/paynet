use node_client::UnspecifiedEnum;
use nuts::{Amount, nut01::PublicKey, nut02::KeysetId};
use rusqlite::Connection;
use thiserror::Error;
use tracing::{error, info};

use crate::{StoreNewProofsError, db, node::RefreshNodeKeysetError, seed_phrase};

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
    AcknowledgeNodeResponse(&'static str, #[source] cashu_client::CashuClientError),
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
    #[error(transparent)]
    SetProofsToState(#[from] db::proof::SetProofsToStateError),
    #[error(transparent)]
    GetProofsByIds(#[from] db::proof::GetProofsByIdsError),
    #[error(transparent)]
    UpdateWadStatusError(#[from] db::wad::UpdateWadStatusError),
    #[error("failed to interact with the node: {0}")]
    CashuClient(#[from] cashu_client::CashuClientError),
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
) -> Result<(), Error> {
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
) -> Result<(), Error> {
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
