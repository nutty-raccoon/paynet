use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use node::{MintQuoteRequest, MintRequest};

/// Seconds since unix epoch
pub fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Hash MintRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_mint_request(request: MintRequest) -> String {
    let mut hasher = DefaultHasher::new();

    for output in &request.outputs {
        output.amount.hash(&mut hasher);
        output.keyset_id.hash(&mut hasher);
        output.blinded_secret.hash(&mut hasher);
    }

    let hash = hasher.finish();

    format!("{:x}", hash)
}

/// Hash MintQuoteRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_mint_quote_request(request: MintQuoteRequest) -> String {
    let mut hasher = DefaultHasher::new();

    request.method.hash(&mut hasher);
    request.amount.hash(&mut hasher);
    request.unit.hash(&mut hasher);
    request.description.hash(&mut hasher);

    let hash = hasher.finish();
    format!("{:x}", hash)
}
