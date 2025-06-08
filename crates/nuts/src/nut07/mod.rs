//! NUT-07: Token state check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofState {
    Unspent,
    Spent,
}

impl ProofState {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ProofState::Unspent),
            1 => Some(ProofState::Spent),
            _ => None,
        }
    }
}

impl From<i32> for ProofState {
    fn from(value: i32) -> Self {
        ProofState::from_i32(value).unwrap_or(ProofState::Unspent)
    }
}

impl From<ProofState> for i32 {
    fn from(state: ProofState) -> Self {
        match state {
            ProofState::Unspent => 0,
            ProofState::Spent => 1,
        }
    }
}

pub struct ProofCheckState {
    pub y: String,
    pub state: ProofState,
    pub witness: Option<String>,
}

pub struct PostCheckStateResponse {
    pub proof_check_states: Vec<ProofCheckState>,
}
