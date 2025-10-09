use std::str::FromStr;

use node_client::{
    AcknowledgeRequest, GetKeysRequest, NodeClient, QuoteStateRequest, RestoreRequest,
};
use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut03::{SwapRequest, SwapResponse},
    nut04::{self, MintQuoteResponse, MintQuoteState, MintRequest},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::{CheckStateResponse, ProofCheckState},
};
use tonic::transport::Channel;
use tonic_types::StatusExt;

use crate::{
    CashuClient, CashuClientError, ClientKey, ClientKeysResponse, ClientKeyset, ClientKeysetKeys,
    ClientKeysetsResponse, ClientMeltQuoteRequest, ClientMeltQuoteResponse, ClientMintQuoteRequest,
    ClientRestoreResponse, Error, NodeInfoResponse,
    proof_errors_handler::{ProofError, ProofErrorKind, extract_proof_index},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
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
}

impl From<Error> for CashuClientError {
    fn from(value: Error) -> Self {
        match value {
            Error::Grpc(status) => {
                if let Some(bad_request) = status.get_error_details().bad_request() {
                    let mut spent = Vec::new();
                    let mut invalid = Vec::new();
                    for violation in &bad_request.field_violations {
                        let idx = extract_proof_index(&violation.field).unwrap_or(0);
                        if violation.description.contains("already spent") {
                            spent.push(idx);
                        } else if violation
                            .description
                            .contains("failed cryptographic verification")
                        {
                            invalid.push(idx);
                        }
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
                } else if status.message() == "inactive keyset"
                    && status.code() == tonic::Code::FailedPrecondition
                {
                    let error_details = status.get_error_details();
                    if let Some(precondition_failure) = error_details.precondition_failure() {
                        for failure in &precondition_failure.violations {
                            if failure.r#type == "keyset.state" {
                                return CashuClientError::InactiveKeyset;
                            }
                        }
                    }
                }

                CashuClientError::Other(Box::new(status))
            }

            e => CashuClientError::Other(Box::new(e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GrpcClient {
    pub node: NodeClient<Channel>,
}

#[async_trait::async_trait]
impl CashuClient for GrpcClient {
    type InnerError = Error;

    async fn keysets(&mut self) -> Result<ClientKeysetsResponse, CashuClientError> {
        let resp = self
            .node
            .keysets(node_client::GetKeysetsRequest {})
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        Ok(ClientKeysetsResponse {
            keysets: resp
                .keysets
                .into_iter()
                .map(|k| ClientKeyset {
                    id: k.id,
                    unit: k.unit,
                    active: k.active,
                })
                .collect(),
        })
    }

    async fn keys(
        &mut self,
        keyset_id: Option<KeysetId>,
    ) -> Result<ClientKeysResponse, CashuClientError> {
        let keys_request = GetKeysRequest {
            keyset_id: keyset_id.map(|id| id.to_bytes().to_vec()),
        };
        let resp = self
            .node
            .keys(keys_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        let keys_response = ClientKeysResponse {
            keysets: resp
                .keysets
                .into_iter()
                .map(|k| -> Result<ClientKeysetKeys, Error> {
                    Ok(ClientKeysetKeys {
                        id: k.id,
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

    async fn mint_quote(
        &mut self,
        req: ClientMintQuoteRequest,
    ) -> Result<MintQuoteResponse<String>, CashuClientError> {
        let mint_quote_request = node_client::MintQuoteRequest {
            method: req.method,
            amount: req.amount,
            unit: req.unit,
            description: req.description,
        };
        let resp = self
            .node
            .mint_quote(mint_quote_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        let mint_quote_response = MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(resp.state).map_err(|_e| {
                    Error::InvalidState {
                        method: "Mint_quote".to_string(),
                    }
                })?,
            )
            .map_err(|_e| Error::InvalidState {
                method: "Mint_quote".to_string(),
            })?,
            expiry: resp.expiry,
        };
        Ok(mint_quote_response)
    }

    async fn mint(
        &mut self,
        req: MintRequest<String>,
        method: String,
    ) -> Result<nuts::nut04::MintResponse, CashuClientError> {
        let mint_request = node_client::MintRequest {
            method,
            quote: req.quote,
            outputs: req
                .outputs
                .into_iter()
                .map(|o| node_client::BlindedMessage {
                    amount: o.amount.into(),
                    keyset_id: o.keyset_id.to_bytes().to_vec(),
                    blinded_secret: o.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        };
        let resp = self
            .node
            .mint(mint_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        let signatures = resp
            .signatures
            .into_iter()
            .map(|s| -> Result<nuts::nut00::BlindSignature, Error> {
                Ok(nuts::nut00::BlindSignature {
                    amount: s.amount.into(),
                    keyset_id: KeysetId::from_bytes(&s.keyset_id).map_err(Error::KeysetId)?,
                    c: PublicKey::from_slice(&s.blind_signature).map_err(Error::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mint_response = nuts::nut04::MintResponse { signatures };
        Ok(mint_response)
    }

    async fn mint_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<MintQuoteResponse<String>, CashuClientError> {
        let resp = self
            .node
            .mint_quote_state(QuoteStateRequest { method, quote })
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        let mint_quote_response = MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(resp.state).map_err(|_e| {
                    Error::InvalidState {
                        method: "mint_quote_state".to_string(),
                    }
                })?,
            )
            .map_err(|_e| Error::InvalidState {
                method: "mint_quote_state".to_string(),
            })?,
            expiry: resp.expiry,
        };
        Ok(mint_quote_response)
    }

    async fn swap(&mut self, req: SwapRequest) -> Result<SwapResponse, CashuClientError> {
        let swap_request = node_client::SwapRequest {
            inputs: req
                .inputs
                .into_iter()
                .map(|p| node_client::Proof {
                    amount: p.amount.into(),
                    keyset_id: p.keyset_id.to_bytes().to_vec(),
                    secret: p.secret.to_string(),
                    unblind_signature: p.c.to_bytes().to_vec(),
                })
                .collect(),
            outputs: req
                .outputs
                .into_iter()
                .map(|bm| node_client::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        };

        let resp = self
            .node
            .swap(swap_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        let swap_response = SwapResponse {
            signatures: resp
                .signatures
                .into_iter()
                .map(
                    |s| -> Result<nuts::nut00::BlindSignature, CashuClientError> {
                        Ok(nuts::nut00::BlindSignature {
                            amount: s.amount.into(),
                            keyset_id: KeysetId::from_bytes(&s.keyset_id)
                                .map_err(Error::KeysetId)?,
                            c: PublicKey::from_slice(&s.blind_signature)
                                .map_err(Error::PublicKey)?,
                        })
                    },
                )
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(swap_response)
    }

    async fn melt_quote(
        &mut self,
        req: ClientMeltQuoteRequest,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError> {
        let melt_quote_request = node_client::MeltQuoteRequest {
            method: req.method,
            unit: req.unit,
            request: req.request,
        };

        let resp = self
            .node
            .melt_quote(melt_quote_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        let melt_quote_response = crate::ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: resp.amount.into(),
            unit: resp.unit,
            expiry: resp.expiry,
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    Error::InvalidState {
                        method: "melt_quote".to_string(),
                    }
                })?,
            )
            .map_err(|_e| Error::InvalidState {
                method: "melt_quote".to_string(),
            })?,
            transfer_ids: Some(resp.transfer_ids),
        };

        Ok(melt_quote_response)
    }

    async fn melt(
        &mut self,
        method: String,
        req: MeltRequest<String>,
    ) -> Result<MeltResponse, CashuClientError> {
        let melt_request = node_client::MeltRequest {
            method,
            quote: req.quote,
            inputs: req
                .inputs
                .into_iter()
                .map(|p| node_client::Proof {
                    amount: p.amount.into(),
                    keyset_id: p.keyset_id.to_bytes().to_vec(),
                    secret: p.secret.to_string(),
                    unblind_signature: p.c.to_bytes().to_vec(),
                })
                .collect(),
        };

        let resp = self
            .node
            .melt(melt_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        let melt_response = MeltResponse {
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    Error::InvalidState {
                        method: "melt".to_string(),
                    }
                })?,
            )
            .map_err(|_e| Error::InvalidState {
                method: "melt".to_string(),
            })?,
            transfer_ids: Some(resp.transfer_ids),
        };
        Ok(melt_response)
    }

    async fn melt_quote_state(
        &mut self,
        method: String,
        quote: String,
    ) -> Result<ClientMeltQuoteResponse, CashuClientError> {
        let resp = self
            .node
            .melt_quote_state(node_client::MeltQuoteStateRequest { method, quote })
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();

        let melt_quote_response = crate::ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: resp.amount.into(),
            unit: resp.unit,
            expiry: resp.expiry,
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    Error::InvalidState {
                        method: "melt_quote_state".to_string(),
                    }
                })?,
            )
            .map_err(|_e| Error::InvalidState {
                method: "melt_quote_state".to_string(),
            })?,
            transfer_ids: Some(resp.transfer_ids),
        };
        Ok(melt_quote_response)
    }

    async fn info(&mut self) -> Result<NodeInfoResponse, CashuClientError> {
        let resp = self
            .node
            .get_node_info(node_client::GetNodeInfoRequest {})
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        Ok(NodeInfoResponse { info: resp.info })
    }

    async fn check_state(
        &mut self,
        req: crate::CheckStateRequest,
    ) -> Result<CheckStateResponse, CashuClientError> {
        let check_state_request = node_client::CheckStateRequest { ys: req.ys };
        let resp = self
            .node
            .check_state(check_state_request)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        let check_state_resp = CheckStateResponse {
            proof_check_states: resp
                .states
                .into_iter()
                .map(|s| -> Result<ProofCheckState, Error> {
                    Ok(ProofCheckState {
                        y: PublicKey::from_slice(&s.y).map_err(Error::PublicKey)?,
                        state: s.state.into(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(check_state_resp)
    }

    async fn acknowledge(
        &mut self,
        path: String,
        request_hash: u64,
    ) -> Result<(), CashuClientError> {
        let _ = self
            .node
            .acknowledge(AcknowledgeRequest { path, request_hash })
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        Ok(())
    }

    async fn restore(
        &mut self,
        outputs: Vec<BlindedMessage>,
    ) -> Result<ClientRestoreResponse, CashuClientError> {
        let node_restore_req = RestoreRequest {
            outputs: outputs
                .into_iter()
                .map(|bm| node_client::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        };

        let resp = self
            .node
            .restore(node_restore_req)
            .await
            .map_err(|e| CashuClientError::from(Error::Grpc(e)))?
            .into_inner();
        Ok(ClientRestoreResponse {
            outputs: resp
                .outputs
                .into_iter()
                .map(|bm| {
                    Ok(BlindedMessage {
                        amount: bm.amount.into(),
                        keyset_id: KeysetId::from_bytes(&bm.keyset_id).map_err(Error::KeysetId)?,
                        blinded_secret: PublicKey::from_slice(&bm.blinded_secret)
                            .map_err(Error::PublicKey)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
            signatures: resp
                .signatures
                .into_iter()
                .map(|s| {
                    Ok(BlindSignature {
                        amount: s.amount.into(),
                        keyset_id: KeysetId::from_bytes(&s.keyset_id).map_err(Error::KeysetId)?,
                        c: PublicKey::from_slice(&s.blind_signature).map_err(Error::PublicKey)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}
