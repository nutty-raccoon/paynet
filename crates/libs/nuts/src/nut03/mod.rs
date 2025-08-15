use serde::{Deserialize, Serialize};

use crate::nut00::{BlindSignature, BlindedMessage, Proofs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapResponse {
    pub signatures: Vec<BlindSignature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    pub inputs: Proofs,
    pub outputs: Vec<BlindedMessage>,
}
