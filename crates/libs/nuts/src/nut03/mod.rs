use serde::{Deserialize, Serialize};

use crate::nut00::{BlindSignature, BlindedMessage, Proofs};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SwapResponse {
    pub signatures: Vec<BlindSignature>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapRequest {
    pub inputs: Proofs,
    pub outputs: Vec<BlindedMessage>,
}
