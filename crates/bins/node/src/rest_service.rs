use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use nuts::{
    Amount,
    nut00::{BlindedMessage, Proofs},
    nut01::PublicKey,
    nut03::{SwapRequest, SwapResponse},
    nut04::{MintQuoteState, MintRequest, MintResponse},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut06::{ContactInfo, NodeInfo, NodeVersion},
    nut07::CheckStateResponse,
    nut19::{Route, hash_melt_request, hash_mint_request, hash_swap_request},
};
use serde::{Deserialize, Serialize};
use signer::GetRootPubKeyRequest;
use starknet_types::Unit;
use std::{fmt::Debug, str::FromStr};
use tonic::Request;
use uuid::Uuid;

use crate::{
    app_state::{AppState, KeysetKeys},
    methods::Method,
    response_cache::{CachedResponse, ResponseCache},
};

/// HTTP error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// Cashu NUT-01: Get keysets
#[derive(Serialize)]
struct GetKeysetsResponse {
    keysets: Vec<Keyset>,
}

#[derive(Serialize)]
struct Keyset {
    id: String,
    unit: String,
    active: bool,
}

/// Cashu NUT-01: Get keys for keysets
#[derive(Serialize)]
struct GetKeysResponse {
    keysets: Vec<KeysetKeys>,
}

/// Cashu NUT-07: Check state request
#[derive(Debug, Serialize, Deserialize)]
struct CheckStateRequest {
    #[serde(rename = "Ys")]
    ys: Vec<String>,
}

#[derive(Serialize)]
struct ProofState {
    #[serde(rename = "Y")]
    y: String,
    state: String,
}

/// NUT-02: Get keysets
async fn get_keysets(
    State(app_state): State<AppState>,
) -> Result<Json<GetKeysetsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut conn = app_state.pg_pool.acquire().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let keysets = db_node::keyset::get_keysets(&mut conn)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .map(|(id, unit, active)| Keyset {
            id: hex::encode(id),
            unit,
            active,
        })
        .collect();

    Ok(Json(GetKeysetsResponse { keysets }))
}

/// NUT-01: Get keys
/// /v1/keys
async fn get_keys(
    State(app_state): State<AppState>,
) -> Result<Json<GetKeysResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut db_conn = app_state.pg_pool.acquire().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let keysets = app_state
        .inner_keys_no_keyset_id(&mut db_conn)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(GetKeysResponse { keysets }))
}

/// NUT-01: Get keys
/// /v1/:keyset_id
async fn get_keys_by_id(
    State(app_state): State<AppState>,
    Path(keyset_id): Path<String>,
) -> Result<Json<GetKeysResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut db_conn = app_state.pg_pool.acquire().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let keyset_id_bytes = hex::decode(&keyset_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid keyset_id: {}", e),
            }),
        )
    })?;
    let keysets = app_state
        .inner_keys_for_keyset_id(&mut db_conn, keyset_id_bytes)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(GetKeysResponse { keysets }))
}

/// NUT-03: Swap tokens
/// /v1/swap
async fn swap(
    State(app_state): State<AppState>,
    Json(request): Json<SwapRequest>,
) -> Result<Json<SwapResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Swap, hash_swap_request(&request));

    // Try to get from cache first
    if let Some(CachedResponse::Swap(swap_response)) = app_state.get_cached_response(&cache_key) {
        return Ok(Json(swap_response));
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
    let inputs = request.inputs;

    let outputs = request.outputs;

    let promises = app_state.inner_swap(&inputs, &outputs).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let swap_response = SwapResponse {
        signatures: promises,
    };

    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Swap(swap_response.clone()))
    {
        tracing::warn!("Failed to cache swap response: {}", e);
    }

    Ok(Json(swap_response))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestMintQuoteRequest {
    pub amount: u64,
    pub unit: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestMintQuoteResponse {
    pub quote: String,
    pub request: String,
    pub state: MintQuoteState,
    pub expiry: u64,
}

/// NUT-04: Mint Tokens
/// /v1/mint/quote
async fn mint_quote(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<RestMintQuoteRequest>,
) -> Result<Json<RestMintQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let amount = Amount::from(request.amount);
    let unit = Unit::from_str(&request.unit).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let response = app_state
        .inner_mint_quote(method, amount, unit)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let mint_quote_response = RestMintQuoteResponse {
        quote: response.quote.to_string(),
        request: response.request.clone(),
        state: response.state,
        expiry: response.expiry,
    };

    Ok(Json(mint_quote_response))
}

/// NUT-04: Mint Tokens
/// /v1/mint/
async fn mint(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<MintRequest<String>>,
) -> Result<Json<MintResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Mint, hash_mint_request(&request));
    if let Some(CachedResponse::Mint(mint_response)) = app_state.get_cached_response(&cache_key) {
        return Ok(Json(mint_response));
    }
    if request.outputs.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Too many outputs: maximum allowed is 64".to_string(),
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
    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let quote_uuid = Uuid::from_str(&request.quote).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let signatures = app_state
        .inner_mint(method, quote_uuid, &request.outputs)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let mint_response = MintResponse {
        signatures: signatures.clone(),
    };
    if let Err(e) = app_state.cache_response(cache_key, CachedResponse::Mint(mint_response.clone()))
    {
        tracing::warn!("Failed to cache mint response: {}", e);
    }

    Ok(Json(mint_response))
}

async fn mint_quote_state(
    State(app_state): State<AppState>,
    Path((method, quote_id)): Path<(String, String)>,
) -> Result<Json<RestMintQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let quote_id = Uuid::from_str(&quote_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let response = app_state
        .inner_mint_quote_state(method, quote_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .unwrap();

    Ok(Json(RestMintQuoteResponse {
        quote: response.quote.to_string(),
        request: response.request,
        state: response.state,
        expiry: response.expiry,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
struct RestMeltQuoteRequest {
    pub request: String,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RestMeltQuoteResponse {
    pub quote: String,
    pub amount: u64,
    pub unit: String,
    pub state: MeltQuoteState,
    pub expiry: u64,
    pub transfer_ids: Option<Vec<String>>,
}

async fn melt_quote(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<RestMeltQuoteRequest>,
) -> Result<Json<RestMeltQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let unit = Unit::from_str(&request.unit).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let response = app_state
        .inner_melt_quote(method, unit, request.request)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(RestMeltQuoteResponse {
        quote: response.quote.to_string(),
        unit: response.unit.to_string(),
        amount: response.amount.into(),
        state: response.state.into(),
        expiry: response.expiry,
        transfer_ids: response.transfer_ids,
    }))
}

async fn melt(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Json(request): Json<MeltRequest<String>>,
) -> Result<Json<MeltResponse>, (StatusCode, Json<ErrorResponse>)> {
    let cache_key = (Route::Melt, hash_melt_request(&request));
    if let Some(CachedResponse::Melt(melt_response)) = app_state.get_cached_response(&cache_key) {
        return Ok(Json(melt_response));
    }

    if request.inputs.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Too many inputs: maximum allowed is 64".to_string(),
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

    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let quote_uuid = Uuid::from_str(&request.quote).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let response = app_state
        .inner_melt(method, quote_uuid, &request.inputs)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    app_state
        .cache_response(cache_key, CachedResponse::Melt(response.clone()))
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;
    Ok(Json(response))
}

async fn melt_quote_state(
    State(app_state): State<AppState>,
    Path(method): Path<String>,
    Path(quote_id): Path<String>,
) -> Result<Json<RestMeltQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let method = Method::from_str(&method).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let quote_id = Uuid::from_str(&quote_id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let response = app_state
        .inner_melt_quote_state(method, quote_id)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(RestMeltQuoteResponse {
        quote: response.quote.to_string(),
        unit: response.unit.to_string(),
        amount: response.amount.into(),
        state: response.state.into(),
        expiry: response.expiry,
        transfer_ids: response.transfer_ids,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeInfoResponse {
    pub info: String,
}

async fn info(
    State(app_state): State<AppState>,
) -> Result<Json<NodeInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let nuts_config = {
        let nuts_read_lock = app_state.nuts.read().await;
        nuts_read_lock.clone()
    };
    let pub_key = app_state
        .signer
        .clone()
        .get_root_pub_key(Request::new(GetRootPubKeyRequest {}))
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .into_inner()
        .root_pubkey;
    let node_info = NodeInfo {
        name: Some("Paynet Test Node".to_string()),
        pubkey: Some(PublicKey::from_str(&pub_key).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?),
        version: Some(NodeVersion {
            name: "some_name".to_string(),
            version: "0.0.0".to_string(),
        }),
        description: Some("A test node".to_string()),
        description_long: Some("This is a longer description of the test node.".to_string()),
        contact: Some(vec![ContactInfo {
            method: "some_method".to_string(),
            info: "some_info".to_string(),
        }]),
        nuts: nuts_config,
        icon_url: Some("http://example.com/icon.png".to_string()),
        urls: Some(vec!["http://example.com".to_string()]),
        motd: Some("Welcome to the node!".to_string()),
        time: Some(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()),
    };

    let node_info_str = serde_json::to_string(&node_info).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    Ok(Json(NodeInfoResponse {
        info: node_info_str,
    }))
}

async fn checkstate(
    State(app_state): State<AppState>,
    Json(request): Json<CheckStateRequest>,
) -> Result<Json<CheckStateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ys = request
        .ys
        .iter()
        .map(|y| PublicKey::from_str(&y).map_err(|_| "Parse Error"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;
    let proof_state = app_state
        .inner_check_state(ys)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .proof_check_states;

    Ok(Json(CheckStateResponse {
        proof_check_states: proof_state,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AcknowledgeRequest {
    pub path: String,
    pub request_hash: u64,
}

async fn acknowledge(
    State(app_state): State<AppState>,
    Json(request): Json<AcknowledgeRequest>,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let path_str: String = request.path.clone();
    let path: Route = Route::from_str(&path_str).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;
    let request_hash = request.request_hash;

    let cache_key = (path, request_hash);

    // check if the request is already in the cache, if so, remove it
    let exist = app_state.response_cache.get(&cache_key);
    if exist.is_some() {
        app_state.response_cache.remove(&cache_key);
    }
    Ok(())
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/v1/keys", get(get_keys))
        .route("/v1/keys/{keyset_id}", get(get_keys_by_id))
        .route("/v1/keysets", get(get_keysets))
        .route("/v1/swap", post(swap))
        .route("/v1/mint/quote/{method}", post(mint_quote))
        .route("/v1/mint/{method}", post(mint))
        .route("/v1/mint/quote/{method}/{quote_id}", get(mint_quote_state))
        .route("/v1/melt/quote/{method}", post(melt_quote))
        .route("/v1/melt/{method}", post(melt))
        .route("/v1/melt/quote/{method}/{quote_id}", get(melt_quote_state))
        .route("/v1/info", get(info))
        .route("/v1/checkstate", post(checkstate))
        .route("/v1/acknowledge", post(acknowledge))
}
