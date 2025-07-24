//! NUT-20: Signature on Mint Quote
//! Optional support for requiring a signature when redeeming a mint quote.

use serde::{Deserialize, Serialize};

/// Settings for NUT20
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Settings {
    pub supported: bool,
}