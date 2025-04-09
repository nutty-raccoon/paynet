use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use node::{MeltRequest, MintQuoteRequest, MintRequest};

/// Seconds since unix epoch
pub fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Hash MintRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_mint_request(request: &MintRequest) -> String {
    let mut hasher = DefaultHasher::new();

    for output in &request.outputs {
        output.amount.hash(&mut hasher);
        output.keyset_id.hash(&mut hasher);
        output.blinded_secret.hash(&mut hasher);
    }

    hasher.finish().to_string()
}

/// Hash MintQuoteRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_mint_quote_request(request: &MintQuoteRequest) -> String {
    let mut hasher = DefaultHasher::new();

    request.method.hash(&mut hasher);
    request.amount.hash(&mut hasher);
    request.unit.hash(&mut hasher);
    request.description.hash(&mut hasher);

    hasher.finish().to_string()
}

/// Hash MeltRequest to a string
/// This is used to create a unique identifier for the request
pub fn hash_melt_request(request: &MeltRequest) -> String {
    let mut hasher = DefaultHasher::new();

    request.method.hash(&mut hasher);
    request.unit.hash(&mut hasher);
    request.request.hash(&mut hasher);
    for input in &request.inputs {
        input.amount.hash(&mut hasher);
        input.keyset_id.hash(&mut hasher);
        input.secret.hash(&mut hasher);
        input.unblind_signature.hash(&mut hasher);
    }

    hasher.finish().to_string()
}
