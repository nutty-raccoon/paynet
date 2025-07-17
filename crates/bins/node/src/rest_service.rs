#[cfg(feature = "rest")]
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};

#[cfg(feature = "rest")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rest")]
use nuts::{
    Amount,
    nut00::{BlindedMessage, Proof, secret::Secret},
    nut01::PublicKey,
    nut02::KeysetId,
    nut19::Route,
};

#[cfg(feature = "rest")]
use std::str::FromStr;

#[cfg(feature = "rest")]
use uuid::Uuid;

#[cfg(feature = "rest")]
use crate::{app_state::AppState, response_cache::CachedResponse};

#[cfg(feature = "rest")]
use node::{
    BlindSignature as GrpcBlindSignature, KeysetKeys as GrpcKeysetKeys,
    SwapResponse as GrpcSwapResponse,
};

#[cfg(feature = "rest")]
use starknet_types::Unit;

/// HTTP error response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Cashu NUT-01: Get keysets
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct GetKeysetsResponse {
    keysets: Vec<KeysetInfo>,
}

#[cfg(feature = "rest")]
#[derive(Serialize)]
struct KeysetInfo {
    id: String,
    unit: String,
    active: bool,
}

/// Cashu NUT-01: Get keys for keysets
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct GetKeysResponse {
    keysets: Vec<KeysetKeys>,
}

#[cfg(feature = "rest")]
#[derive(Serialize)]
struct KeysetKeys {
    id: String,
    unit: String,
    active: bool,
    keys: std::collections::HashMap<u64, String>,
}

/// Cashu NUT-03: Swap request
#[cfg(feature = "rest")]
#[derive(Deserialize, Debug)]
struct SwapRequest {
    inputs: Vec<ProofHttp>,
    outputs: Vec<BlindedMessageHttp>,
}

/// Cashu NUT-03: Swap response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct SwapResponse {
    signatures: Vec<BlindSignatureHttp>,
}

/// HTTP representation of Proof
#[cfg(feature = "rest")]
#[derive(Deserialize, Debug)]
struct ProofHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    secret: String,
    #[serde(rename = "C")]
    c: String,
}

/// HTTP representation of BlindedMessage
#[cfg(feature = "rest")]
#[derive(Deserialize, Debug)]
struct BlindedMessageHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    #[serde(rename = "B_")]
    blinded_secret: String,
}

/// HTTP representation of BlindSignature
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct BlindSignatureHttp {
    amount: u64,
    #[serde(rename = "id")]
    keyset_id: String,
    #[serde(rename = "C_")]
    c: String,
}

/// Cashu NUT-04: Mint quote request
#[cfg(feature = "rest")]
#[derive(Deserialize)]
struct MintQuoteRequest {
    unit: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// Cashu NUT-04: Mint quote response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct MintQuoteResponse {
    quote: String,
    request: String,
    state: String,
    expiry: u64,
}

/// Cashu NUT-04: Mint request
#[cfg(feature = "rest")]
#[derive(Deserialize, Debug)]
struct MintRequest {
    quote: String,
    outputs: Vec<BlindedMessageHttp>,
}

/// Cashu NUT-04: Mint response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct MintResponse {
    signatures: Vec<BlindSignatureHttp>,
}

/// Cashu NUT-05: Melt request
#[cfg(feature = "rest")]
#[derive(Deserialize, Debug)]
struct MeltRequest {
    quote: String,
    inputs: Vec<ProofHttp>,
}

/// Cashu NUT-05: Melt response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct MeltResponse {
    quote: String,
    amount: u64,
    fee: u64,
    state: String,
    expiry: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    transfer_ids: Vec<String>,
}

/// Cashu NUT-06: Node info response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct NodeInfoResponse {
    // This will be the JSON string from the gRPC response
    #[serde(flatten)]
    info: serde_json::Value,
}

/// Cashu NUT-07: Check state request
#[cfg(feature = "rest")]
#[derive(Deserialize)]
struct CheckStateRequest {
    #[serde(rename = "Ys")]
    ys: Vec<String>,
}

/// Cashu NUT-07: Check state response
#[cfg(feature = "rest")]
#[derive(Serialize)]
struct CheckStateResponse {
    states: Vec<ProofState>,
}

#[cfg(feature = "rest")]
#[derive(Serialize)]
struct ProofState {
    #[serde(rename = "Y")]
    y: String,
    state: String,
}

#[cfg(feature = "rest")]
fn hash_swap_request_http(request: &SwapRequest) -> u64 {
    // For now, use a simple hash - we should match the gRPC implementation
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "rest")]
fn hash_mint_request_http(request: &MintRequest) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

#[cfg(feature = "rest")]
fn hash_melt_request_http(request: &MeltRequest) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{:?}", request).hash(&mut hasher);
    hasher.finish()
}

/// Convert HTTP proof to internal Proof
#[cfg(feature = "rest")]
fn convert_proof_from_http(proof_http: ProofHttp) -> Result<Proof, String> {
    Ok(Proof {
        amount: Amount::from(proof_http.amount),
        keyset_id: KeysetId::from_str(&proof_http.keyset_id)
            .map_err(|e| format!("Invalid keyset_id: {}", e))?,
        secret: Secret::new(proof_http.secret).map_err(|e| format!("Invalid secret: {}", e))?,
        c: PublicKey::from_str(&proof_http.c).map_err(|e| format!("Invalid signature: {}", e))?,
    })
}

/// Convert HTTP blinded message to internal BlindedMessage
#[cfg(feature = "rest")]
fn convert_blinded_message_from_http(
    bm_http: BlindedMessageHttp,
) -> Result<BlindedMessage, String> {
    Ok(BlindedMessage {
        amount: Amount::from(bm_http.amount),
        keyset_id: KeysetId::from_str(&bm_http.keyset_id)
            .map_err(|e| format!("Invalid keyset_id: {}", e))?,
        blinded_secret: PublicKey::from_str(&bm_http.blinded_secret)
            .map_err(|e| format!("Invalid blinded_secret: {}", e))?,
    })
}

/// NUT-01: Get keysets
#[cfg(feature = "rest")]
async fn get_keysets(
    State(app_state): State<AppState>,
) -> Result<Json<GetKeysetsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = app_state.pg_pool.acquire().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let keysets = db_node::keyset::get_keysets(&mut conn)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .map(|(id, unit, active)| KeysetInfo {
            id: hex::encode(id),
            unit,
            active,
        })
        .collect();

    Ok(Json(GetKeysetsResponse { keysets }))
}

/// NUT-01: Get keys
#[cfg(feature = "rest")]
async fn get_keys(
    State(app_state): State<AppState>,
    keyset_id: Option<Path<String>>,
) -> Result<Json<GetKeysResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut db_conn = app_state.pg_pool.acquire().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let keysets = match keyset_id {
        Some(Path(keyset_id_str)) => {
            let keyset_id_bytes = hex::decode(&keyset_id_str).map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Invalid keyset_id: {}", e),
                    }),
                )
            })?;
            app_state
                .inner_keys_for_keyset_id(&mut db_conn, keyset_id_bytes)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: e.to_string(),
                        }),
                    )
                })?
        }
        None => app_state
            .inner_keys_no_keyset_id(&mut db_conn)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )
            })?,
    };

    let response_keysets = keysets
        .into_iter()
        .map(|keyset| {
            let keys = keyset
                .keys
                .into_iter()
                .map(|key| (key.amount, key.pubkey))
                .collect();

            KeysetKeys {
                id: hex::encode(keyset.id),
                unit: keyset.unit,
                active: keyset.active,
                keys,
            }
        })
        .collect();

    Ok(Json(GetKeysResponse {
        keysets: response_keysets,
    }))
}

/// NUT-03: Swap tokens
#[cfg(feature = "rest")]
async fn swap(
    State(app_state): State<AppState>,
    Json(request): Json<SwapRequest>,
) -> Result<Json<SwapResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Swap, hash_swap_request_http(&request));

    // Try to get from cache first
    if let Some(CachedResponse::Swap(swap_response)) = app_state.get_cached_response(&cache_key) {
        let http_response = SwapResponse {
            signatures: swap_response
                .signatures
                .into_iter()
                .map(|sig| BlindSignatureHttp {
                    amount: sig.amount,
                    keyset_id: hex::encode(sig.keyset_id),
                    c: hex::encode(sig.blind_signature),
                })
                .collect(),
        };
        return Ok(Json(http_response));
    }

    if request.inputs.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Too many inputs: maximum allowed is 64".to_string(),
            }),
        ));
    }
    if request.outputs.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Too many outputs: maximum allowed is 64".to_string(),
            }),
        ));
    }
    if request.inputs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Inputs cannot be empty".to_string(),
            }),
        ));
    }
    if request.outputs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Outputs cannot be empty".to_string(),
            }),
        ));
    }

    // Convert HTTP types to internal types
    let inputs = request
        .inputs
        .into_iter()
        .map(convert_proof_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let outputs = request
        .outputs
        .into_iter()
        .map(convert_blinded_message_from_http)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;

    let promises = app_state.inner_swap(&inputs, &outputs).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let signatures = promises
        .iter()
        .map(|p| BlindSignatureHttp {
            amount: p.amount.into(),
            keyset_id: hex::encode(p.keyset_id.to_bytes()),
            c: hex::encode(p.c.to_bytes()),
        })
        .collect();

    let swap_response = SwapResponse { signatures };

    // Store in cache (convert back to gRPC format for caching)
    let grpc_response = GrpcSwapResponse {
        signatures: promises
            .iter()
            .map(|p| GrpcBlindSignature {
                amount: p.amount.into(),
                keyset_id: p.keyset_id.to_bytes().to_vec(),
                blind_signature: p.c.to_bytes().to_vec(),
            })
            .collect(),
    };

    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Swap(grpc_response)) {
        tracing::warn!("Failed to cache swap response: {}", e);
    }

    Ok(Json(swap_response))
}

#[cfg(feature = "rest")]
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/v1/keys", get(get_keys))
        .route(
            "/v1/keys/:keyset_id",
            get(
                |State(app_state): State<AppState>, Path(keyset_id): Path<String>| async move {
                    get_keys(State(app_state), Some(Path(keyset_id))).await
                },
            ),
        )
        .route("/v1/swap", post(swap))
    // TODO: Add remaining endpoints for mint, melt, info, checkstate
}
