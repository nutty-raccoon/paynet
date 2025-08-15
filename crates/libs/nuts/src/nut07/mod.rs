//! NUT-07: Token state check

use serde::{Deserialize, Serialize};

use crate::nut01::PublicKey;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofState {
    Unspecified,
    Unspent,
    Pending,
    Spent,
}

impl ProofState {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ProofState::Unspecified),
            1 => Some(ProofState::Unspent),
            2 => Some(ProofState::Pending),
            3 => Some(ProofState::Spent),
            _ => None,
        }
    }
}

impl From<i32> for ProofState {
    fn from(value: i32) -> Self {
        ProofState::from_i32(value).unwrap_or(ProofState::Unspecified)
    }
}

impl From<ProofState> for i32 {
    fn from(state: ProofState) -> Self {
        match state {
            ProofState::Unspecified => 0,
            ProofState::Unspent => 1,
            ProofState::Pending => 2,
            ProofState::Spent => 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofCheckState {
    pub y: PublicKey,
    pub state: ProofState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckStateResponse {
    pub proof_check_states: Vec<ProofCheckState>,
}
