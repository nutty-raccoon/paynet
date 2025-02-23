use crate::{Error, keyset_cache::CachedKeysetInfo};
use std::{str::FromStr, sync::Arc};

use node::{
    BlindSignature, GetKeysetsRequest, GetKeysetsResponse, Keyset, MeltRequest, MeltResponse,
    MintQuoteRequest, MintQuoteResponse, MintRequest, MintResponse, Node, QuoteStateRequest,
    SwapRequest, SwapResponse,
};
use nuts::{
    Amount, QuoteTTLConfig,
    nut00::{BlindedMessage, Proof, secret::Secret},
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut06::NutsSettings,
};
use sqlx::PgPool;
use starknet_types::{MeltPaymentRequest, Unit};
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status, transport::Channel};
use uuid::Uuid;

use crate::{
    app_state::{NutsSettingsState, QuoteTTLConfigState, SharedSignerClient},
    keyset_cache::KeysetCache,
    methods::Method,
};

#[derive(Debug)]
pub struct GrpcState {
    pub pg_pool: PgPool,
    pub signer: SharedSignerClient,
    pub keyset_cache: KeysetCache,
    pub nuts: NutsSettingsState,
    pub quote_ttl: Arc<QuoteTTLConfigState>,
    // TODO: add a cache for the mint_quote and melt routes
}

impl GrpcState {
    pub fn new(
        pg_pool: PgPool,
        signer_client: signer::SignerClient<Channel>,
        nuts_settings: NutsSettings<Method, Unit>,
        quote_ttl: QuoteTTLConfig,
    ) -> Self {
        Self {
            pg_pool,
            keyset_cache: Default::default(),
            nuts: Arc::new(RwLock::new(nuts_settings)),
            quote_ttl: Arc::new(quote_ttl.into()),
            signer: Arc::new(RwLock::new(signer_client)),
        }
    }

    pub async fn init_first_keysets(
        &self,
        method: Method,
        units: &[Unit],
        index: u32,
        max_order: u32,
    ) -> Result<(), Error> {
        let mut insert_keysets_query_builder = db_node::InsertKeysetsQueryBuilder::new();

        for unit in units {
            let keyset_id = {
                let mut signer_lock = self.signer.write().await;
                let response = signer_lock
                    .declare_keyset(signer::DeclareKeysetRequest {
                        method: method.to_string(),
                        unit: unit.to_string(),
                        index,
                        max_order,
                    })
                    .await?;
                KeysetId::from_bytes(&response.into_inner().keyset_id)?
            };

            insert_keysets_query_builder.add_row(keyset_id, unit, max_order, index);
            self.keyset_cache
                .insert(keyset_id, CachedKeysetInfo::new(true, *unit))
                .await;
        }

        let mut conn = self.pg_pool.acquire().await?;
        insert_keysets_query_builder.execute(&mut conn).await?;

        Ok(())
    }
}

#[derive(Debug, Error)]
enum ParseGrpcError {
    #[error(transparent)]
    KeysetId(nut02::Error),
    #[error(transparent)]
    PublicKey(nut01::Error),
    #[error(transparent)]
    Unit(starknet_types::UnitFromStrError),
    #[error(transparent)]
    Method(crate::methods::FromStrError),
    #[error(transparent)]
    Uuid(uuid::Error),
    #[error(transparent)]
    MeltPayment(serde_json::Error),
}

impl From<ParseGrpcError> for Status {
    fn from(value: ParseGrpcError) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

#[tonic::async_trait]
impl Node for GrpcState {
    async fn keysets(
        &self,
        _request: Request<GetKeysetsRequest>,
    ) -> Result<Response<GetKeysetsResponse>, Status> {
        let mut conn = self
            .pg_pool
            .acquire()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let keysets = db_node::get_keysets(&mut conn)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .map(|(id, unit, active)| Keyset {
                id: id.to_vec(),
                unit,
                active,
            })
            .collect();

        Ok(Response::new(GetKeysetsResponse { keysets }))
    }

    async fn swap(
        &self,
        swap_request: Request<SwapRequest>,
    ) -> Result<Response<SwapResponse>, Status> {
        let swap_request = swap_request.into_inner();

        let inputs = swap_request
            .inputs
            .into_iter()
            .map(|p| -> Result<Proof, ParseGrpcError> {
                Ok(Proof {
                    amount: p.amount.into(),
                    keyset_id: KeysetId::from_bytes(&p.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    secret: Secret::new(p.secret),
                    c: PublicKey::from_slice(&p.unblind_signature)
                        .map_err(ParseGrpcError::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let outputs = swap_request
            .outputs
            .into_iter()
            .map(|bm| -> Result<BlindedMessage, ParseGrpcError> {
                Ok(BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: KeysetId::from_bytes(&bm.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    blinded_secret: PublicKey::from_slice(&bm.blinded_secret)
                        .map_err(ParseGrpcError::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let promises = self.inner_swap(&inputs, &outputs).await?;

        Ok(Response::new(SwapResponse {
            signatures: promises
                .iter()
                .map(|p| BlindSignature {
                    amount: p.amount.into(),
                    keyset_id: p.keyset_id.to_bytes().to_vec(),
                    blind_signature: p.c.to_bytes().to_vec(),
                })
                .collect(),
        }))
    }

    async fn mint_quote(
        &self,
        mint_quote_request: Request<MintQuoteRequest>,
    ) -> Result<Response<MintQuoteResponse>, Status> {
        let mint_quote_request = mint_quote_request.into_inner();

        let method =
            Method::from_str(&mint_quote_request.method).map_err(ParseGrpcError::Method)?;
        let amount = Amount::from(mint_quote_request.amount);
        let unit = Unit::from_str(&mint_quote_request.unit).map_err(ParseGrpcError::Unit)?;

        let response = self.inner_mint_quote(method, amount, unit).await?;

        Ok(Response::new(MintQuoteResponse {
            quote: response.quote.to_string(),
            request: response.request,
            state: node::MintQuoteState::from(response.state).into(),
            expiry: response.expiry,
        }))
    }

    async fn mint(
        &self,
        mint_request: Request<MintRequest>,
    ) -> Result<Response<MintResponse>, Status> {
        let mint_request = mint_request.into_inner();

        let method = Method::from_str(&mint_request.method).map_err(ParseGrpcError::Method)?;
        let quote_id = Uuid::from_str(&mint_request.quote).map_err(ParseGrpcError::Uuid)?;
        let outputs = mint_request
            .outputs
            .into_iter()
            .map(|bm| -> Result<BlindedMessage, ParseGrpcError> {
                Ok(BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: KeysetId::from_bytes(&bm.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    blinded_secret: PublicKey::from_slice(&bm.blinded_secret)
                        .map_err(ParseGrpcError::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let promises = self.inner_mint(method, quote_id, &outputs).await?;

        Ok(Response::new(MintResponse {
            signatures: promises
                .iter()
                .map(|p| BlindSignature {
                    amount: p.amount.into(),
                    keyset_id: p.keyset_id.to_bytes().to_vec(),
                    blind_signature: p.c.to_bytes().to_vec(),
                })
                .collect(),
        }))
    }

    async fn melt(
        &self,
        melt_request: Request<MeltRequest>,
    ) -> Result<Response<MeltResponse>, Status> {
        let melt_request = melt_request.into_inner();

        let method = Method::from_str(&melt_request.method).map_err(ParseGrpcError::Method)?;
        let unit = Unit::from_str(&melt_request.unit).map_err(ParseGrpcError::Unit)?;
        let melt_payment_request: MeltPaymentRequest =
            serde_json::from_str(&melt_request.request).map_err(ParseGrpcError::MeltPayment)?;
        let inputs = melt_request
            .inputs
            .into_iter()
            .map(|p| -> Result<Proof, ParseGrpcError> {
                Ok(Proof {
                    amount: p.amount.into(),
                    keyset_id: KeysetId::from_bytes(&p.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    secret: Secret::new(p.secret),
                    c: PublicKey::from_slice(&p.unblind_signature)
                        .map_err(ParseGrpcError::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let response = self
            .inner_melt(method, unit, melt_payment_request, &inputs)
            .await?;

        Ok(Response::new(MeltResponse {
            quote: response.quote.to_string(),
            amount: response.amount.into(),
            fee: response.fee.into(),
            state: node::MeltState::from(response.state).into(),
            expiry: response.expiry,
        }))
    }

    async fn mint_quote_state(
        &self,
        mint_quote_state_request: Request<QuoteStateRequest>,
    ) -> Result<Response<MintQuoteResponse>, Status> {
        let mint_quote_state_request = mint_quote_state_request.into_inner();
        let method =
            Method::from_str(&mint_quote_state_request.method).map_err(ParseGrpcError::Method)?;
        let quote_id =
            Uuid::from_str(&mint_quote_state_request.quote).map_err(ParseGrpcError::Uuid)?;

        let response = self.inner_mint_quote_state(method, quote_id).await?;

        Ok(Response::new(MintQuoteResponse {
            quote: response.quote.to_string(),
            request: response.request,
            state: node::MintQuoteState::from(response.state).into(),
            expiry: response.expiry,
        }))
    }

    async fn melt_quote_state(
        &self,
        melt_quote_state_request: Request<QuoteStateRequest>,
    ) -> Result<Response<MeltResponse>, Status> {
        let melt_quote_state_request = melt_quote_state_request.into_inner();
        let method =
            Method::from_str(&melt_quote_state_request.method).map_err(ParseGrpcError::Method)?;
        let quote_id =
            Uuid::from_str(&melt_quote_state_request.quote).map_err(ParseGrpcError::Uuid)?;

        let response = self.inner_melt_quote_state(method, quote_id).await?;

        Ok(Response::new(MeltResponse {
            quote: response.quote.to_string(),
            amount: response.amount.into(),
            fee: response.fee.into(),
            state: node::MeltState::from(response.state).into(),
            expiry: response.expiry,
        }))
    }
}
