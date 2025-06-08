use crate::{
    keyset_cache::CachedKeysetInfo,
    liquidity_sources::LiquiditySources,
    response_cache::{CachedResponse, InMemResponseCache, ResponseCache},
};
use node::{
    AcknowledgeRequest, AcknowledgeResponse, BlindSignature, GetKeysRequest, GetKeysResponse,
    GetKeysetsRequest, GetKeysetsResponse, GetNodeInfoRequest, Keyset, MeltRequest, MeltResponse,
    MintQuoteRequest, MintQuoteResponse, MintRequest, MintResponse, Node, NodeInfoResponse,
    QuoteStateRequest, SwapRequest, SwapResponse, hash_melt_request, hash_mint_request,
    hash_swap_request,
};
use nuts::{
    Amount, QuoteTTLConfig,
    nut00::{BlindedMessage, Proof, secret::Secret},
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut06::{ContactInfo, NodeInfo, NodeVersion, NutsSettings},
    nut19::{CacheResponseKey, Route},
};
use signer::GetRootPubKeyRequest;
use sqlx::PgPool;
use starknet_types::Unit;
use std::{str::FromStr, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    app_state::{NutsSettingsState, QuoteTTLConfigState, SignerClient},
    keyset_cache::KeysetCache,
    methods::Method,
};

#[derive(Debug, Clone)]
pub struct GrpcState {
    pub pg_pool: PgPool,
    pub signer: SignerClient,
    pub keyset_cache: KeysetCache,
    pub nuts: NutsSettingsState,
    pub quote_ttl: Arc<QuoteTTLConfigState>,
    pub liquidity_sources: LiquiditySources,
    pub response_cache: Arc<InMemResponseCache<(Route, u64), CachedResponse>>,
}

#[derive(Debug, thiserror::Error)]
pub enum InitKeysetError {
    #[error(transparent)]
    Tonic(#[from] tonic::Status),
    #[error(transparent)]
    Nut01(#[from] nut01::Error),
    #[error(transparent)]
    Nut02(#[from] nut02::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl GrpcState {
    pub fn new(
        pg_pool: PgPool,
        signer_client: SignerClient,
        nuts_settings: NutsSettings<Method, Unit>,
        quote_ttl: QuoteTTLConfig,
        liquidity_sources: LiquiditySources,
    ) -> Self {
        Self {
            pg_pool,
            keyset_cache: Default::default(),
            nuts: Arc::new(RwLock::new(nuts_settings)),
            quote_ttl: Arc::new(quote_ttl.into()),
            signer: signer_client,
            liquidity_sources,
            response_cache: Arc::new(InMemResponseCache::new(None)),
        }
    }

    pub async fn init_first_keysets(
        &self,
        units: &[Unit],
        index: u32,
        max_order: u32,
    ) -> Result<(), InitKeysetError> {
        let mut insert_keysets_query_builder = db_node::InsertKeysetsQueryBuilder::new();

        for unit in units {
            let response = self
                .signer
                .clone()
                .declare_keyset(signer::DeclareKeysetRequest {
                    unit: unit.to_string(),
                    index,
                    max_order,
                })
                .await?;
            let response = response.into_inner();
            let keyset_id = KeysetId::from_bytes(&response.keyset_id)?;

            insert_keysets_query_builder.add_row(keyset_id, unit, max_order, index);

            self.keyset_cache
                .insert_info(keyset_id, CachedKeysetInfo::new(true, *unit, max_order))
                .await;

            let keys = response
                .keys
                .into_iter()
                .map(|k| -> Result<(Amount, PublicKey), InitKeysetError> {
                    Ok((
                        Amount::from(k.amount),
                        PublicKey::from_str(&k.pubkey).map_err(InitKeysetError::Nut01)?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            self.keyset_cache
                .insert_keys(keyset_id, keys.into_iter())
                .await;
        }

        let mut conn = self.pg_pool.acquire().await?;
        insert_keysets_query_builder.execute(&mut conn).await?;

        Ok(())
    }

    pub fn get_cached_response(&self, cache_key: &CacheResponseKey) -> Option<CachedResponse> {
        if let Some(cached_response) = self.response_cache.get(cache_key) {
            return Some(cached_response);
        }

        None
    }

    pub fn cache_response(
        &self,
        cache_key: (Route, u64),
        response: CachedResponse,
    ) -> Result<(), Status> {
        self.response_cache
            .insert(cache_key, response)
            .map_err(|e| Status::internal(e.to_string()))?;

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
    Secret(nuts::nut00::secret::Error),
}

impl From<ParseGrpcError> for Status {
    fn from(value: ParseGrpcError) -> Self {
        Status::invalid_argument(value.to_string())
    }
}

#[tonic::async_trait]
impl Node for GrpcState {
    #[instrument]
    async fn keysets(
        &self,
        _request: Request<GetKeysetsRequest>,
    ) -> Result<Response<GetKeysetsResponse>, Status> {
        let mut conn = self
            .pg_pool
            .acquire()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let keysets = db_node::keyset::get_keysets(&mut conn)
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

    #[instrument]
    async fn keys(
        &self,
        request: Request<GetKeysRequest>,
    ) -> Result<Response<GetKeysResponse>, Status> {
        let request = request.into_inner();

        let mut db_conn = self
            .pg_pool
            .acquire()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let keysets = match request.keyset_id {
            Some(keyset_id) => {
                self.inner_keys_with_keyset_id(&mut db_conn, keyset_id)
                    .await?
            }
            None => self.inner_keys_no_keyset_id(&mut db_conn).await?,
        };

        Ok(Response::new(GetKeysResponse { keysets }))
    }

    #[instrument]
    async fn swap(
        &self,
        swap_request: Request<SwapRequest>,
    ) -> Result<Response<SwapResponse>, Status> {
        let swap_request = swap_request.into_inner();

        let cache_key = (Route::Swap, hash_swap_request(&swap_request));
        // Try to get from cache first
        if let Some(CachedResponse::Swap(swap_response)) = self.get_cached_response(&cache_key) {
            return Ok(Response::new(swap_response));
        }

        if swap_request.inputs.len() > 64 {
            return Err(Status::invalid_argument(
                "Too many inputs: maximum allowed is 64",
            ));
        }
        if swap_request.outputs.len() > 64 {
            return Err(Status::invalid_argument(
                "Too many outputs: maximum allowed is 64",
            ));
        }

        if swap_request.inputs.is_empty() {
            return Err(Status::invalid_argument("Inputs cannot be empty"));
        }
        if swap_request.outputs.is_empty() {
            return Err(Status::invalid_argument("Outputs cannot be empty"));
        }

        let inputs = swap_request
            .inputs
            .into_iter()
            .map(|p| -> Result<Proof, ParseGrpcError> {
                Ok(Proof {
                    amount: p.amount.into(),
                    keyset_id: KeysetId::from_bytes(&p.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    secret: Secret::new(p.secret).map_err(ParseGrpcError::Secret)?,
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

        let swap_response = SwapResponse {
            signatures: promises
                .iter()
                .map(|p| BlindSignature {
                    amount: p.amount.into(),
                    keyset_id: p.keyset_id.to_bytes().to_vec(),
                    blind_signature: p.c.to_bytes().to_vec(),
                })
                .collect(),
        };

        // Store in cache
        self.cache_response(cache_key, CachedResponse::Swap(swap_response.clone()))?;

        Ok(Response::new(swap_response))
    }

    #[instrument]
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

        let mint_quote_response = MintQuoteResponse {
            quote: response.quote.to_string(),
            request: response.request.clone(),
            state: node::MintQuoteState::from(response.state).into(),
            expiry: response.expiry,
        };

        Ok(Response::new(mint_quote_response))
    }

    #[instrument]
    async fn mint(
        &self,
        mint_request: Request<MintRequest>,
    ) -> Result<Response<MintResponse>, Status> {
        let mint_request = mint_request.into_inner();

        let cache_key = (Route::Mint, hash_mint_request(&mint_request));
        // Try to get from cache first
        if let Some(CachedResponse::Mint(mint_response)) = self.get_cached_response(&cache_key) {
            return Ok(Response::new(mint_response));
        }

        if mint_request.outputs.len() > 64 {
            return Err(Status::invalid_argument(
                "Too many outputs: maximum allowed is 64",
            ));
        }

        let method = Method::from_str(&mint_request.method).map_err(ParseGrpcError::Method)?;

        if mint_request.outputs.is_empty() {
            return Err(Status::invalid_argument("Outputs cannot be empty"));
        }

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
        let signatures = promises
            .iter()
            .map(|p| BlindSignature {
                amount: p.amount.into(),
                keyset_id: p.keyset_id.to_bytes().to_vec(),
                blind_signature: p.c.to_bytes().to_vec(),
            })
            .collect::<Vec<_>>();

        let mint_response = MintResponse {
            signatures: signatures.clone(),
        };

        // Store in cache
        self.cache_response(cache_key, CachedResponse::Mint(mint_response.clone()))?;

        Ok(Response::new(mint_response))
    }

    #[instrument]
    async fn melt(
        &self,
        melt_request: Request<MeltRequest>,
    ) -> Result<Response<MeltResponse>, Status> {
        let melt_request = melt_request.into_inner();

        let cache_key = (Route::Melt, hash_melt_request(&melt_request));

        // Try to get from cache first
        if let Some(CachedResponse::Melt(melt_response)) = self.get_cached_response(&cache_key) {
            return Ok(Response::new(melt_response));
        }

        if melt_request.inputs.len() > 64 {
            return Err(Status::invalid_argument(
                "Too many inputs: maximum allowed is 64",
            ));
        }

        if melt_request.inputs.is_empty() {
            return Err(Status::invalid_argument("Inputs cannot be empty"));
        }

        let method = Method::from_str(&melt_request.method).map_err(ParseGrpcError::Method)?;
        let unit = Unit::from_str(&melt_request.unit).map_err(ParseGrpcError::Unit)?;
        let inputs = melt_request
            .clone()
            .inputs
            .into_iter()
            .map(|p| -> Result<Proof, ParseGrpcError> {
                Ok(Proof {
                    amount: p.amount.into(),
                    keyset_id: KeysetId::from_bytes(&p.keyset_id)
                        .map_err(ParseGrpcError::KeysetId)?,
                    secret: Secret::new(p.secret).map_err(ParseGrpcError::Secret)?,
                    c: PublicKey::from_slice(&p.unblind_signature)
                        .map_err(ParseGrpcError::PublicKey)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let response = self
            .inner_melt(method, unit, melt_request.request, &inputs)
            .await?;

        let melt_response = MeltResponse {
            quote: response.quote.to_string(),
            amount: response.amount.into(),
            fee: response.fee.into(),
            state: node::MeltState::from(response.state).into(),
            expiry: response.expiry,
            transfer_id: response.transfer_id,
        };

        // Store in cache
        self.cache_response(cache_key, CachedResponse::Melt(melt_response.clone()))?;

        Ok(Response::new(melt_response))
    }

    #[instrument]
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

    #[instrument]
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
            transfer_id: response.transfer_id,
        }))
    }

    #[instrument]
    async fn get_node_info(
        &self,
        _node_info_request: Request<GetNodeInfoRequest>,
    ) -> Result<Response<NodeInfoResponse>, Status> {
        let nuts_config = {
            let nuts_read_lock = self.nuts.read().await;
            nuts_read_lock.clone()
        };
        let pub_key = self
            .signer
            .clone()
            .get_root_pub_key(Request::new(GetRootPubKeyRequest {}))
            .await?
            .into_inner()
            .root_pubkey;
        let node_info = NodeInfo {
            name: Some("Paynet Test Node".to_string()),
            pubkey: Some(
                PublicKey::from_str(&pub_key).map_err(|e| Status::internal(e.to_string()))?,
            ),
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

        let node_info_str =
            serde_json::to_string(&node_info).map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(NodeInfoResponse {
            info: node_info_str,
        }))
    }

    /// acknowledge is for the client to say he successfully stored the quote_id
    #[instrument]
    async fn acknowledge(
        &self,
        ack_request: Request<AcknowledgeRequest>,
    ) -> Result<Response<AcknowledgeResponse>, Status> {
        let path_str: String = ack_request.get_ref().path.clone();
        let path: Route =
            Route::from_str(&path_str).map_err(|_| Status::invalid_argument("Invalid path"))?;
        let request_hash = ack_request.get_ref().request_hash;

        let cache_key = (path, request_hash);

        // check if the request is already in the cache, if so, remove it
        let exist = self.response_cache.get(&cache_key);
        if exist.is_some() {
            self.response_cache.remove(&cache_key);
        }

        Ok(Response::new(AcknowledgeResponse {}))
    }
}
