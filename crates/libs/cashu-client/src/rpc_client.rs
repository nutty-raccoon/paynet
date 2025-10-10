use std::str::FromStr;

use nuts::{
    Amount,
    nut00::BlindedMessage,
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut03::{SwapRequest, SwapResponse},
    nut04::{self, MintQuoteResponse, MintQuoteState, MintRequest, MintResponse},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::CheckStateResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CashuClient, CashuClientError, ClientKey, ClientKeysResponse, ClientKeyset, ClientKeysetKeys,
    ClientKeysetsResponse, ClientMeltQuoteRequest, ClientMeltQuoteResponse, ClientMintQuoteRequest,
    ClientRestoreResponse, NodeInfoResponse,
    proof_errors_handler::{ProofError, ProofErrorKind},
};

/// RPC client error types
#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid State in {method}")]
    InvalidState { method: String },
    #[error(transparent)]
    KeysetId(nut02::Error),
    #[error(transparent)]
    PublicKey(nut01::Error),
    #[error(transparent)]
    Method(nut04::Error),
    #[error("invalid field format: '[' or ']' not found")]
    InvalidFormat,
    #[error("invalid index: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
    /// Server returned error response
    #[error("Server error: {0}")]
    Server(String),
}

/// Convert RPC Error to CashuClientError
impl From<Error> for CashuClientError {
    fn from(value: Error) -> Self {
        match value {
            Error::Http(e) => CashuClientError::Other(Box::new(e)),
            Error::Server(msg) => {
                if msg.contains("inactive keyset") {
                    return CashuClientError::InactiveKeyset;
                }
                if msg.contains("quote not found") {
                    return CashuClientError::QuoteNotFound;
                }
                if msg.contains("already spent")
                    || msg.contains("failed cryptographic verification")
                {
                    let mut spent = Vec::new();
                    let mut invalid = Vec::new();
                    if msg.contains("already spent") {
                        spent.push(0);
                    }
                    if msg.contains("failed cryptographic verification") {
                        invalid.push(0);
                    }
                    let errs = vec![
                        ProofError {
                            indexes: spent,
                            kind: ProofErrorKind::AlreadySpent,
                        },
                        ProofError {
                            indexes: invalid,
                            kind: ProofErrorKind::FailCryptoVerify,
                        },
                    ];
                    return CashuClientError::Proof(errs);
                }
                CashuClientError::Other(Box::new(Error::Server(msg)))
            }
            e => CashuClientError::Other(Box::new(e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcClient {
    pub url: String,
    client: reqwest::Client,
}

impl RpcClient {
    /// Create a new RPC client with the given base URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }
}

/// REST API response types
/// These match the structures defined in rest_service.rs

#[derive(Serialize, Deserialize)]
struct RestKeysetsResponse {
    keysets: Vec<RestKeyset>,
}

#[derive(Serialize, Deserialize)]
struct RestKeyset {
    id: String,
    unit: String,
    active: bool,
}

#[derive(Serialize, Deserialize)]
struct RestKeysResponse {
    keysets: Vec<RestKeysetKeys>,
}

#[derive(Serialize, Deserialize)]
struct RestKeysetKeys {
    id: String,
    unit: String,
    active: bool,
    keys: Vec<RestKeyAmount>,
}

#[derive(Serialize, Deserialize)]
struct RestKeyAmount {
    amount: u64,
    pubkey: String,
}

/// REST-specific mint quote request (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestMintQuoteRequest {
    amount: u64,
    unit: String,
    description: Option<String>,
}

/// REST-specific mint quote response (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestMintQuoteResponse {
    quote: String,
    request: String,
    state: MintQuoteState,
    expiry: u64,
}

/// REST-specific melt quote request (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestMeltQuoteRequest {
    request: String,
    unit: String,
}

/// REST-specific melt quote response (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestMeltQuoteResponse {
    quote: String,
    amount: u64,
    unit: String,
    state: MeltQuoteState,
    expiry: u64,
    transfer_ids: Option<Vec<String>>,
}

/// REST-specific melt response (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestMeltResponse {
    state: MeltQuoteState,
    transfer_ids: Option<Vec<String>>,
}

/// REST-specific check state request (matches rest_service.rs)
#[derive(Debug, Serialize, Deserialize)]
struct RestCheckStateRequest {
    #[serde(rename = "Ys")]
    ys: Vec<String>,
}

/// REST-specific acknowledge request (matches rest_service.rs)
#[derive(Serialize, Deserialize, Debug)]
struct RestAcknowledgeRequest {
    path: String,
    request_hash: u64,
}

/// REST-specific node info response (matches rest_service.rs)
#[derive(Serialize, Deserialize)]
struct RestNodeInfoResponse {
    info: String,
}

/// REST-specific error response (matches rest_service.rs)
#[derive(Serialize, Deserialize)]
struct RestErrorResponse {
    error: String,
}

#[async_trait::async_trait]
impl CashuClient for RpcClient {
    type InnerError = Error;

    /// Get all keysets from the mint
    /// Route: GET /v1/keysets
    async fn keysets(&mut self) -> Result<ClientKeysetsResponse, CashuClientError> {
        let url = format!("{}/v1/keysets", self.url);
        let response = self.client.get(&url).send().await.map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestKeysetsResponse = response.json().await.map_err(Error::Http)?;

        Ok(ClientKeysetsResponse {
            keysets: resp
                .keysets
                .into_iter()
                .map(|k| ClientKeyset {
                    id: hex::decode(&k.id).unwrap_or_default(),
                    unit: k.unit,
                    active: k.active,
                })
                .collect(),
        })
    }

    /// Get keys for a specific keyset or all keysets
    /// Route: GET /v1/keys or GET /v1/keys/{keyset_id}
    async fn keys(
        &mut self,
        keyset_id: Option<KeysetId>,
    ) -> Result<ClientKeysResponse, CashuClientError> {
        let url = if let Some(id) = keyset_id {
            format!("{}/v1/keys/{}", self.url, hex::encode(id.to_bytes()))
        } else {
            format!("{}/v1/keys", self.url)
        };

        let response = self.client.get(&url).send().await.map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestKeysResponse = response.json().await.map_err(Error::Http)?;

        let keys_response = ClientKeysResponse {
            keysets: resp
                .keysets
                .into_iter()
                .map(|k| -> Result<ClientKeysetKeys, Error> {
                    Ok(ClientKeysetKeys {
                        id: hex::decode(&k.id)?,
                        unit: k.unit,
                        active: k.active,
                        keys: k
                            .keys
                            .into_iter()
                            .map(|key| -> Result<ClientKey, Error> {
                                Ok(ClientKey {
                                    amount: Amount::from(key.amount),
                                    publickey: PublicKey::from_str(&key.pubkey)
                                        .map_err(Error::PublicKey)?,
                                })
                            })
                            .collect::<Result<Vec<ClientKey>, Error>>()?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
        };

        Ok(keys_response)
    }

    /// Request a mint quote
    /// Route: POST /v1/mint/quote/{method}
    async fn mint_quote(
        &mut self,
        req: ClientMintQuoteRequest,
    ) -> Result<MintQuoteResponse<String>, CashuClientError> {
        let url = format!("{}/v1/mint/quote/{}", self.url, req.method);

        let request = RestMintQuoteRequest {
            amount: req.amount,
            unit: req.unit,
            description: req.description,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestMintQuoteResponse = response.json().await.map_err(Error::Http)?;

        // Convert REST response to nuts MintQuoteResponse
        Ok(MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: resp.state,
            expiry: resp.expiry,
        })
    }

    /// Mint tokens
    /// Route: POST /v1/mint/{method}
    async fn mint(
        &mut self,
        req: MintRequest<String>,
        method: String,
    ) -> Result<MintResponse, CashuClientError> {
        let url = format!("{}/v1/mint/{}", self.url, method);

        let response = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        // Response is already in the correct MintResponse format from nuts
        let resp: MintResponse = response.json().await.map_err(Error::Http)?;

        Ok(resp)
    }

    /// Get mint quote state
    /// Route: GET /v1/mint/quote/{method}/{quote_id}
    async fn mint_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<MintQuoteResponse<String>, CashuClientError> {
        let url = format!("{}/v1/mint/quote/{}/{}", self.url, method, quote);

        let response = self.client.get(&url).send().await.map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestMintQuoteResponse = response.json().await.map_err(Error::Http)?;

        // Convert REST response to nuts MintQuoteResponse
        Ok(MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: resp.state,
            expiry: resp.expiry,
        })
    }

    /// Swap tokens
    /// Route: POST /v1/swap
    async fn swap(&mut self, req: SwapRequest) -> Result<SwapResponse, CashuClientError> {
        let url = format!("{}/v1/swap", self.url);

        let response = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        // Response is already in the correct SwapResponse format from nuts
        let resp: SwapResponse = response.json().await.map_err(Error::Http)?;

        Ok(resp)
    }

    /// Request a melt quote
    /// Route: POST /v1/melt/quote/{method}
    async fn melt_quote(
        &mut self,
        req: ClientMeltQuoteRequest,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError> {
        let url = format!("{}/v1/melt/quote/{}", self.url, req.method);

        let request = RestMeltQuoteRequest {
            request: req.request,
            unit: req.unit,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestMeltQuoteResponse = response.json().await.map_err(Error::Http)?;

        // Convert REST response to ClientMeltQuoteResponse
        Ok(ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: Amount::from(resp.amount),
            unit: resp.unit,
            state: resp.state,
            expiry: resp.expiry,
            transfer_ids: resp.transfer_ids,
        })
    }

    /// Melt tokens
    /// Route: POST /v1/melt/{method}
    async fn melt(
        &mut self,
        method: String,
        req: MeltRequest<String>,
    ) -> Result<MeltResponse, CashuClientError> {
        let url = format!("{}/v1/melt/{}", self.url, method);

        let response = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        // Deserialize REST response and convert to MeltResponse
        let resp: RestMeltResponse = response.json().await.map_err(Error::Http)?;

        Ok(MeltResponse {
            state: resp.state,
            transfer_ids: resp.transfer_ids,
        })
    }

    /// Get melt quote state
    /// Route: GET /v1/melt/quote/{method}/{quote_id}
    async fn melt_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError> {
        let url = format!("{}/v1/melt/quote/{}/{}", self.url, method, quote);

        let response = self.client.get(&url).send().await.map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestMeltQuoteResponse = response.json().await.map_err(Error::Http)?;

        // Convert REST response to ClientMeltQuoteResponse
        Ok(ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: Amount::from(resp.amount),
            unit: resp.unit,
            state: resp.state,
            expiry: resp.expiry,
            transfer_ids: resp.transfer_ids,
        })
    }

    /// Get node info
    /// Route: GET /v1/info
    async fn info(&mut self) -> Result<NodeInfoResponse, CashuClientError> {
        let url = format!("{}/v1/info", self.url);

        let response = self.client.get(&url).send().await.map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        let resp: RestNodeInfoResponse = response.json().await.map_err(Error::Http)?;

        // Convert REST response to cashu-client NodeInfoResponse
        Ok(NodeInfoResponse { info: resp.info })
    }

    /// Check state of proofs
    /// Route: POST /v1/checkstate
    async fn check_state(
        &mut self,
        req: crate::CheckStateRequest,
    ) -> Result<CheckStateResponse, CashuClientError> {
        let url = format!("{}/v1/checkstate", self.url);

        // Convert bytes to hex strings for the REST API
        let ys: Vec<String> = req.ys.iter().map(hex::encode).collect();

        let request = RestCheckStateRequest { ys };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        // Response is already in the correct CheckStateResponse format from nuts
        let resp: CheckStateResponse = response.json().await.map_err(Error::Http)?;

        Ok(resp)
    }

    /// Acknowledge a cached response
    /// Route: POST /v1/acknowledge
    async fn acknowledge(
        &mut self,
        path: String,
        request_hash: u64,
    ) -> Result<(), CashuClientError> {
        let url = format!("{}/v1/acknowledge", self.url);

        let request = RestAcknowledgeRequest { path, request_hash };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            let error_resp: RestErrorResponse = response.json().await.map_err(Error::Http)?;
            return Err(Error::Server(error_resp.error).into());
        }

        Ok(())
    }

    /// Restore blinded messages
    /// Note: This endpoint is not implemented in rest_service.rs
    /// Returning an error for now
    async fn restore(
        &mut self,
        _outputs: Vec<BlindedMessage>,
    ) -> Result<ClientRestoreResponse, CashuClientError> {
        Err(CashuClientError::Other(Box::new(Error::Server(
            "Restore endpoint not implemented in REST API".to_string(),
        ))))
    }
}
