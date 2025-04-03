//! NUT-07: Token state check

use crate::nut00::Proof;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Token already spent")]
    TokenSpent,
    #[error("Invalid token state")]
    InvalidState,
    #[error("Token verification failed")]
    VerificationFailed,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TokenState {
    #[default]
    Unspent,
    Spent,
    Pending,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStateRequest {
    #[serde(rename = "Ys")]
    pub ys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStateInfo {
    #[serde(rename = "Y")]
    pub y: String,
    pub state: TokenState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckStateResponse {
    pub states: Vec<TokenStateInfo>,
}

impl TokenState {
    pub fn is_spendable(&self) -> bool {
        matches!(self, Self::Unspent)
    }
}

impl FromStr for TokenState {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_uppercase().as_str() {
            "UNSPENT" => Ok(Self::Unspent),
            "SPENT" => Ok(Self::Spent),
            "PENDING" => Ok(Self::Pending),
            "INVALID" => Ok(Self::Invalid),
            _ => Err(Error::InvalidState),
        }
    }
}

pub trait ProofExtensions {
    /// Converts a proof into a state check request containing the Y point necessary
    /// for querying token states from a mint.
    fn to_state_check_request(&self) -> Result<CheckStateRequest, Error>;
}

impl ProofExtensions for Proof {
    fn to_state_check_request(&self) -> Result<CheckStateRequest, Error> {
        let y = self.y().map_err(|_| Error::VerificationFailed)?;
        Ok(CheckStateRequest {
            ys: vec![y.to_hex()],
        })
    }
}

impl ProofExtensions for Vec<Proof> {
    fn to_state_check_request(&self) -> Result<CheckStateRequest, Error> {
        let mut ys = Vec::with_capacity(self.len());

        for proof in self {
            let y = proof.y().map_err(|_| Error::VerificationFailed)?.to_hex();
            ys.push(y);
        }

        Ok(CheckStateRequest { ys })
    }
}

/// Interface for storing and retrieving token states
pub trait TokenStateStore {
    /// Get the current state of a token
    fn get_token_state(&self, y: &str) -> Result<TokenState, Error>;

    /// Get the witness data for a spent token if available
    fn get_token_witness(&self, y: &str) -> Result<Option<String>, Error>;

    fn set_token_state(&mut self, y: &str, state: TokenState) -> Result<(), Error>;

    fn set_token_witness(&mut self, y: &str, witness: String) -> Result<(), Error>;
}

/// In-memory implementation of TokenStateStore for tracking token states and witnesses.
pub struct InMemoryTokenStore {
    states: HashMap<String, TokenState>,
    witnesses: HashMap<String, String>,
}

impl InMemoryTokenStore {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            witnesses: HashMap::new(),
        }
    }
}

impl Default for InMemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenStateStore for InMemoryTokenStore {
    fn get_token_state(&self, y: &str) -> Result<TokenState, Error> {
        Ok(self.states.get(y).cloned().unwrap_or(TokenState::Unspent))
    }

    fn get_token_witness(&self, y: &str) -> Result<Option<String>, Error> {
        Ok(self.witnesses.get(y).cloned())
    }

    fn set_token_state(&mut self, y: &str, state: TokenState) -> Result<(), Error> {
        self.states.insert(y.to_string(), state);
        Ok(())
    }

    fn set_token_witness(&mut self, y: &str, witness: String) -> Result<(), Error> {
        self.witnesses.insert(y.to_string(), witness);
        Ok(())
    }
}

/// Check if a proof has been spent
pub fn is_proof_spent(proof: &Proof, store: &impl TokenStateStore) -> Result<bool, Error> {
    let y = proof.y().map_err(|_| Error::VerificationFailed)?.to_hex();
    let state = store.get_token_state(&y)?;

    Ok(state == TokenState::Spent)
}

/// Handle a state check request from the mint API
pub fn handle_check_state_request(
    request: &CheckStateRequest,
    store: &impl TokenStateStore,
) -> Result<CheckStateResponse, Error> {
    let mut states = Vec::with_capacity(request.ys.len());

    for y in &request.ys {
        let state = store.get_token_state(y)?;
        let witness = if state == TokenState::Spent {
            store.get_token_witness(y)?
        } else {
            None
        };

        states.push(TokenStateInfo {
            y: y.clone(),
            state,
            witness,
        });
    }

    Ok(CheckStateResponse { states })
}

/// Mark a proof as having a specific state
pub fn mark_proof_state(
    proof: &Proof,
    state: TokenState,
    store: &mut impl TokenStateStore,
    witness: Option<String>,
) -> Result<(), Error> {
    let y = proof.y().map_err(|_| Error::VerificationFailed)?.to_hex();
    store.set_token_state(&y, state.clone())?;

    if let (Some(witness_data), TokenState::Spent) = (witness, state) {
        store.set_token_witness(&y, witness_data)?;
    }

    Ok(())
}

/// Helper functions
pub fn mark_proof_spent(
    proof: &Proof,
    store: &mut impl TokenStateStore,
    witness: Option<String>,
) -> Result<(), Error> {
    mark_proof_state(proof, TokenState::Spent, store, witness)
}

pub fn mark_proof_pending(proof: &Proof, store: &mut impl TokenStateStore) -> Result<(), Error> {
    mark_proof_state(proof, TokenState::Pending, store, None)
}

pub fn mark_proof_unspent(proof: &Proof, store: &mut impl TokenStateStore) -> Result<(), Error> {
    mark_proof_state(proof, TokenState::Unspent, store, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Amount;
    use crate::nut00::secret::Secret;
    use crate::nut01::PublicKey;
    use crate::nut02::KeysetId;
    use std::str::FromStr;

    fn create_test_proof(secret_hex: &str) -> Result<Proof, crate::nut00::secret::Error> {
        let secret = Secret::from_str(secret_hex)?;
        Ok(Proof {
            amount: Amount::from(10_u16),
            keyset_id: KeysetId::from_str("00456a94ab4e1c46").unwrap(),
            secret,
            c: PublicKey::from_hex(
                "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
            )
            .unwrap(),
        })
    }

    #[test]
    fn test_token_state_management() {
        let mut store = InMemoryTokenStore::new();
        let proof =
            create_test_proof("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();

        // Initially unspent
        assert!(!is_proof_spent(&proof, &store).unwrap());

        // Mark as spent
        mark_proof_spent(&proof, &mut store, None).unwrap();

        // Should now be spent
        assert!(is_proof_spent(&proof, &store).unwrap());
    }

    #[test]
    fn test_witness_handling() {
        let mut store = InMemoryTokenStore::new();
        let proof =
            create_test_proof("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let witness = r#"{"signatures": ["signature"]}"#;

        mark_proof_spent(&proof, &mut store, Some(witness.to_string())).unwrap();

        let request = proof.to_state_check_request().unwrap();
        let response = handle_check_state_request(&request, &store).unwrap();

        assert_eq!(response.states.len(), 1);
        assert_eq!(response.states[0].state, TokenState::Spent);
        assert_eq!(response.states[0].witness, Some(witness.to_string()));
    }

    #[test]
    fn test_state_transitions() {
        let mut store = InMemoryTokenStore::new();
        let proof =
            create_test_proof("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();

        mark_proof_pending(&proof, &mut store).unwrap();

        let request = proof.to_state_check_request().unwrap();
        let response = handle_check_state_request(&request, &store).unwrap();
        assert_eq!(response.states[0].state, TokenState::Pending);

        mark_proof_unspent(&proof, &mut store).unwrap();

        let response = handle_check_state_request(&request, &store).unwrap();
        assert_eq!(response.states[0].state, TokenState::Unspent);
    }
}
