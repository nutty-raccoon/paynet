use bitcoin::bip32::Xpriv;
use nuts::{
    Amount,
    dhke::{sign_message, verify_message},
    nut01::PublicKey,
    nut02::{KeysetId, MintKeySet},
};
use server_errors::Error;
use signer::{
    DeclareKeysetRequest, DeclareKeysetResponse, GetRootPubKeyRequest, GetRootPubKeyResponse, Key,
    SignBlindedMessagesRequest, SignBlindedMessagesResponse, SignerServer, VerifyProofsRequest,
    VerifyProofsResponse,
};
use state::{SharedKeySetCache, SharedRootKey};
use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::RwLock;
use tonic::{Request, Response, Status, service::LayerExt};
use tower::ServiceBuilder;
use tracing::{instrument, trace};

mod server_errors;
mod state;

const ROOT_KEY_ENV_VAR: &str = "ROOT_KEY";
const GRPC_PORT_ENV_VAR: &str = "GRPC_PORT";

const PROOFS_FIELD: &str = "proofs";
const MESSAGES_FIELD: &str = "messages";

#[derive(Debug)]
pub struct SignerState {
    root_key: SharedRootKey,
    keyset_cache: SharedKeySetCache,
}

#[tonic::async_trait]
impl signer::Signer for SignerState {
    #[instrument]
    async fn declare_keyset(
        &self,
        declare_keyset_request: Request<DeclareKeysetRequest>,
    ) -> Result<Response<DeclareKeysetResponse>, Status> {
        let declare_keyset_request = declare_keyset_request.get_ref();

        // By setting this limit, we make sure that the bigest key has amount 2^63
        // which is exactly i64::MAX. So we can convert any proof amount to i64 safely,
        // this is really usefull for interacting with external dependecies,
        // such as databases or networking protocols.
        if declare_keyset_request.max_order > 64 {
            return Err(Error::MaxOrderTooBig(declare_keyset_request.max_order))?;
        }

        let unit = starknet_types::Unit::from_str(&declare_keyset_request.unit)
            .map_err(|_| Error::UnknownUnit(&declare_keyset_request.unit))?;

        let keyset = {
            let keyset = create_new_starknet_keyset(
                self.root_key.clone(),
                unit,
                declare_keyset_request.index,
                declare_keyset_request
                    .max_order
                    .try_into()
                    .map_err(|_| Error::MaxOrderTooBig(declare_keyset_request.max_order))?,
            );

            self.keyset_cache
                .insert(keyset.id, keyset.keys.clone())
                .await;

            keyset
        };

        Ok(Response::new(DeclareKeysetResponse {
            keyset_id: keyset.id.to_bytes().to_vec(),
            keys: keyset
                .keys
                .iter()
                .map(|(&amout, keypair)| Key {
                    amount: amout.into(),
                    pubkey: keypair.public_key.to_string(),
                })
                .collect(),
        }))
    }

    #[instrument]
    async fn sign_blinded_messages(
        &self,
        sign_blinded_messages_request: Request<SignBlindedMessagesRequest>,
    ) -> Result<Response<SignBlindedMessagesResponse>, Status> {
        let blinded_messages = sign_blinded_messages_request.into_inner().messages;

        let mut signatures = Vec::with_capacity(blinded_messages.len());

        let keyset_cache_read_lock = self.keyset_cache.0.read().await;

        for (idx, blinded_message) in blinded_messages.into_iter().enumerate() {
            let amount = Amount::from(blinded_message.amount);
            if !blinded_message.amount.is_power_of_two() {
                return Err(Error::AmountNotPowerOfTwo(idx, amount))?;
            }
            let keyset_id = KeysetId::from_bytes(&blinded_message.keyset_id).map_err(|e| {
                Error::BadKeysetId(MESSAGES_FIELD, idx, &blinded_message.keyset_id, e)
            })?;

            let keyset = keyset_cache_read_lock
                .get(&keyset_id)
                .ok_or(Error::KeysetNotFound(MESSAGES_FIELD, idx, keyset_id))?;
            let max_order: u64 = keyset
                .last_key_value()
                .map(|(&k, _)| k)
                .unwrap_or_default()
                .into();
            if u64::from(amount) > max_order {
                return Err(Error::AmountGreaterThanMax(
                    idx,
                    amount,
                    Amount::from(max_order),
                ))?;
            }

            let key_pair = {
                keyset.get(&amount).ok_or(Error::AmountNotFound(
                    MESSAGES_FIELD,
                    idx,
                    keyset_id,
                    amount,
                ))?
            };

            let blind_secret = PublicKey::from_slice(&blinded_message.blinded_secret)
                .map_err(|e| Error::BadSecret(idx, e))?;

            let c = sign_message(&key_pair.secret_key, &blind_secret)
                .map_err(|e| Error::CouldNotSignMessage(idx, blind_secret, e))?;

            signatures.push(c.to_bytes().to_vec());
        }

        Ok(Response::new(SignBlindedMessagesResponse { signatures }))
    }

    #[instrument]
    async fn verify_proofs(
        &self,
        verify_proofs_request: Request<VerifyProofsRequest>,
    ) -> Result<Response<VerifyProofsResponse>, Status> {
        let proofs = verify_proofs_request.into_inner().proofs;

        for (idx, proof) in proofs.into_iter().enumerate() {
            let keyset_id = KeysetId::from_bytes(&proof.keyset_id)
                .map_err(|e| Error::BadKeysetId(PROOFS_FIELD, idx, &proof.keyset_id, e))?;
            let amount = Amount::from(proof.amount);
            if !proof.amount.is_power_of_two() {
                return Err(Error::AmountNotPowerOfTwo(idx, amount))?;
            }
            let (secret_key, max_order) = {
                let keyset_cache_read_lock = self.keyset_cache.0.read().await;

                let keyset = keyset_cache_read_lock
                    .get(&keyset_id)
                    .ok_or(Error::KeysetNotFound(PROOFS_FIELD, idx, keyset_id))?;
                let max_order: u64 = keyset
                    .last_key_value()
                    .map(|(&k, _)| k)
                    .unwrap_or_default()
                    .into();

                let keyset = keyset
                    .get(&amount)
                    .ok_or(Error::AmountNotFound(PROOFS_FIELD, idx, keyset_id, amount))?
                    .secret_key
                    .clone();
                (keyset, max_order)
            };

            if u64::from(amount) > max_order {
                return Err(Error::AmountGreaterThanMax(
                    idx,
                    amount,
                    Amount::from(max_order),
                ))?;
            }

            let c = PublicKey::from_slice(&proof.unblind_signature)
                .map_err(|e| Error::InvalidSignature(idx, e))?;

            if !verify_message(&secret_key, c, proof.secret.as_bytes())
                .map_err(|e| Error::CouldNotVerifyProof(idx, c, proof.secret, e))?
            {
                return Ok(Response::new(VerifyProofsResponse { is_valid: false }));
            };
        }

        Ok(Response::new(VerifyProofsResponse { is_valid: true }))
    }

    #[instrument]
    async fn get_root_pub_key(
        &self,
        _get_root_pub_key_request: tonic::Request<GetRootPubKeyRequest>,
    ) -> Result<Response<GetRootPubKeyResponse>, Status> {
        let pub_key = self.root_key.get_pubkey();

        Ok(Response::new(GetRootPubKeyResponse {
            root_pubkey: pub_key.to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    let (meter_provider, subscriber) = open_telemetry_tracing::init(PKG_NAME, PKG_VERSION);
    tracing::subscriber::set_global_default(subscriber).unwrap();
    opentelemetry::global::set_meter_provider(meter_provider);

    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("signer.env")
            .inspect_err(|e| tracing::error!("dotenvy initialization failed: {e}"));
    }

    let socket_addr: SocketAddr = {
        let socket_port_env_var: String =
            std::env::var(GRPC_PORT_ENV_VAR).expect("env var `GRPC_PORT` should be set");
        format!("[::0]:{}", socket_port_env_var).parse()?
    };
    let root_private_key = {
        let root_key_env_var: String =
            std::env::var(ROOT_KEY_ENV_VAR).expect("env var `ROOT_KEY` should be set");
        Xpriv::from_str(&root_key_env_var)
            .expect("content of `ROOT_KEY` env var should be a valid private key")
    };

    let signer_logic = SignerState {
        root_key: SharedRootKey(Arc::new(root_private_key)),
        keyset_cache: SharedKeySetCache(Arc::new(RwLock::new(HashMap::new()))),
    };

    let signer_server_service = ServiceBuilder::new()
        .layer(tower_otel::trace::GrpcLayer::server(tracing::Level::INFO))
        .named_layer(SignerServer::new(signer_logic));

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<SignerServer<SignerState>>()
        .await;

    trace!(name: "grpc-listen", port = socket_addr.port());

    tonic::transport::Server::builder()
        .add_service(signer_server_service)
        .add_service(health_service)
        .serve(socket_addr)
        .await?;

    Ok(())
}

fn create_new_starknet_keyset(
    root_key: SharedRootKey,
    unit: starknet_types::Unit,
    index: u32,
    max_order: u8,
) -> MintKeySet<starknet_types::Unit> {
    root_key.generate_keyset(unit, index, max_order)
}
