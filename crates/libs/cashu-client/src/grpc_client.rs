use node_client::{AcknowledgeRequest, NodeClient, QuoteStateRequest};
use nuts::{
    nut01::PublicKey,
    nut02::KeysetId,
    nut03::{SwapRequest, SwapResponse},
    nut04::{MintQuoteResponse, MintQuoteState, MintRequest},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::{CheckStateResponse, ProofCheckState},
};
use tonic::transport::Channel;

use crate::{
    CashuClient, ClientMeltQuoteRequest, ClientMeltQuoteResponse, ClientMintQuoteRequest, Error,
    NodeInfoResponse,
};

#[derive(Clone)]
pub struct GrpcClient {
    node: NodeClient<Channel>,
}

#[async_trait::async_trait]
impl CashuClient for GrpcClient {
    async fn mint_quote(
        &mut self,
        req: ClientMintQuoteRequest,
    ) -> Result<MintQuoteResponse<String>, Error> {
        let mint_quote_request = node_client::MintQuoteRequest {
            method: req.method,
            amount: req.amount,
            unit: req.unit,
            description: req.description,
        };
        let resp = self.node.mint_quote(mint_quote_request).await?.into_inner();
        let mint_quote_response = MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(resp.state).map_err(|_e| {
                    crate::Error::InvalidState {
                        method: "Mint_quote".to_string(),
                    }
                })?,
            )
            .map_err(|_e| crate::Error::InvalidState {
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
    ) -> Result<nuts::nut04::MintResponse, crate::Error> {
        let mint_request = node_client::MintRequest {
            method: method,
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
        let resp = self.node.mint(mint_request).await?.into_inner();
        let signatures = resp
            .signatures
            .into_iter()
            .map(|s| -> Result<nuts::nut00::BlindSignature, crate::Error> {
                Ok(nuts::nut00::BlindSignature {
                    amount: s.amount.into(),
                    keyset_id: KeysetId::from_bytes(&s.keyset_id)
                        .map_err(crate::Error::KeysetId)?,
                    c: PublicKey::from_slice(&s.blind_signature)
                        .map_err(crate::Error::PublicKey)?,
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
    ) -> Result<MintQuoteResponse<String>, crate::Error> {
        let resp = self
            .node
            .mint_quote_state(QuoteStateRequest { method, quote })
            .await?
            .into_inner();
        let mint_quote_response = MintQuoteResponse {
            quote: resp.quote,
            request: resp.request,
            state: MintQuoteState::try_from(
                node_client::MintQuoteState::try_from(resp.state).map_err(|_e| {
                    crate::Error::InvalidState {
                        method: "mint_quote_state".to_string(),
                    }
                })?,
            )
            .map_err(|_e| crate::Error::InvalidState {
                method: "mint_quote_state".to_string(),
            })?,
            expiry: resp.expiry,
        };
        Ok(mint_quote_response)
    }

    async fn swap(&mut self, req: SwapRequest) -> Result<SwapResponse, crate::Error> {
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

        let resp = self.node.swap(swap_request).await?.into_inner();

        let swap_response = SwapResponse {
            signatures: resp
                .signatures
                .into_iter()
                .map(|s| -> Result<nuts::nut00::BlindSignature, crate::Error> {
                    Ok(nuts::nut00::BlindSignature {
                        amount: s.amount.into(),
                        keyset_id: KeysetId::from_bytes(&s.keyset_id)
                            .map_err(crate::Error::KeysetId)?,
                        c: PublicKey::from_slice(&s.blind_signature)
                            .map_err(crate::Error::PublicKey)?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(swap_response)
    }

    async fn melt_quote(
        &mut self,
        req: ClientMeltQuoteRequest,
    ) -> Result<ClientMeltQuoteResponse, crate::Error> {
        let melt_quote_request = node_client::MeltQuoteRequest {
            method: req.method,
            unit: req.unit,
            request: req.request,
        };

        let resp = self.node.melt_quote(melt_quote_request).await?.into_inner();

        let melt_quote_response = crate::ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: resp.amount.into(),
            unit: resp.unit,
            expiry: resp.expiry,
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    crate::Error::InvalidState {
                        method: "melt_quote".to_string(),
                    }
                })?,
            )
            .map_err(|_e| crate::Error::InvalidState {
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
    ) -> Result<MeltResponse, crate::Error> {
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

        let resp = self.node.melt(melt_request).await?.into_inner();

        let melt_response = MeltResponse {
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    crate::Error::InvalidState {
                        method: "melt".to_string(),
                    }
                })?,
            )
            .map_err(|_e| crate::Error::InvalidState {
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
    ) -> Result<ClientMeltQuoteResponse, crate::Error> {
        let resp = self
            .node
            .melt_quote_state(node_client::MeltQuoteStateRequest { method, quote })
            .await?
            .into_inner();

        let melt_quote_response = crate::ClientMeltQuoteResponse {
            quote: resp.quote,
            amount: resp.amount.into(),
            unit: resp.unit,
            expiry: resp.expiry,
            state: MeltQuoteState::try_from(
                node_client::MeltQuoteState::try_from(resp.state).map_err(|_e| {
                    crate::Error::InvalidState {
                        method: "melt_quote_state".to_string(),
                    }
                })?,
            )
            .map_err(|_e| crate::Error::InvalidState {
                method: "melt_quote_state".to_string(),
            })?,
            transfer_ids: Some(resp.transfer_ids),
        };
        Ok(melt_quote_response)
    }

    async fn info(&mut self) -> Result<NodeInfoResponse, crate::Error> {
        let resp = self
            .node
            .get_node_info(node_client::GetNodeInfoRequest {})
            .await?
            .into_inner();
        Ok(NodeInfoResponse { info: resp.info })
    }

    async fn check_state(
        &mut self,
        req: crate::CheckStateRequest,
    ) -> Result<CheckStateResponse, crate::Error> {
        let check_state_request = node_client::CheckStateRequest { ys: req.ys };
        let resp = self
            .node
            .check_state(check_state_request)
            .await?
            .into_inner();
        let check_state_resp = CheckStateResponse {
            proof_check_states: resp
                .states
                .into_iter()
                .map(|s| -> Result<ProofCheckState, crate::Error> {
                    Ok(ProofCheckState {
                        y: PublicKey::from_slice(&s.y).map_err(crate::Error::PublicKey)?,
                        state: s.state.into(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(check_state_resp)
    }

    async fn acknowledge(&mut self, path: String, request_hash: u64) -> Result<(), crate::Error> {
        let _ = self
            .node
            .acknowledge(AcknowledgeRequest { path, request_hash })
            .await?
            .into_inner();
        Ok(())
    }
}
