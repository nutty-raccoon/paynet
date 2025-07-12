use serde::{Deserialize, Serialize};

/// Secs wuotes are valid
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuoteTTLConfig {
    /// Seconds mint quote is valid
    pub mint_ttl: u64,
    /// Seconds melt quote is valid
    pub melt_ttl: u64,
}

impl QuoteTTLConfig {
    /// Create new [`QuoteTTL`]
    pub fn new(mint_ttl: u64, melt_ttl: u64) -> QuoteTTLConfig {
        Self { mint_ttl, melt_ttl }
    }
}
