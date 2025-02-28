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
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};
use tonic::{Request, Response, Status, transport::Server};

mod server_errors;
mod state;

const ROOT_KEY_ENV_VAR: &str = "ROOT_KEY";
const SOCKET_PORT_ENV_VAR: &str = "SOCKET_PORT";
const SOCKET_IP_ENV_VAR: &str = "SOCKET_IP";

#[derive(Debug)]
pub struct SignerState {
    root_key: SharedRootKey,
    keyset_cache: SharedKeySetCache,
}

#[tonic::async_trait]
impl signer::Signer for SignerState {
    async fn declare_keyset(
        &self,
        declare_keyset_request: Request<DeclareKeysetRequest>,
    ) -> Result<Response<DeclareKeysetResponse>, Status> {
        let declare_keyset_request = declare_keyset_request.get_ref();

        let unit = starknet_types::Unit::from_str(&declare_keyset_request.unit).map_err(|_| {
            Status::invalid_argument(Error::UnknownUnit(&declare_keyset_request.unit).to_string())
        })?;

        let keyset = {
            let keyset = create_new_starknet_keyset(
                self.root_key.clone(),
                unit,
                declare_keyset_request.index,
                declare_keyset_request
                    .max_order
                    .try_into()
                    .map_err(|_| Status::invalid_argument(Error::MaxOrderTooBig.to_string()))?,
            );

            self.keyset_cache
                .insert(keyset.id, keyset.keys.clone())
                .map_err(|e| Status::internal(e.to_string()))?;

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
    async fn sign_blinded_messages(
        &self,
        sign_blinded_messages_request: Request<SignBlindedMessagesRequest>,
    ) -> Result<Response<SignBlindedMessagesResponse>, Status> {
        let blinded_messages = sign_blinded_messages_request.into_inner().messages;

        let mut signatures = Vec::with_capacity(blinded_messages.len());

        let keyset_cache_read_lock = self
            .keyset_cache
            .0
            .read()
            .map_err(|_| Status::internal(Error::LockPoisoned))?;

        for blinded_message in blinded_messages {
            let keyset_id = KeysetId::from_bytes(&blinded_message.keyset_id)
                .map_err(|_| Status::invalid_argument(Error::BadKeysetId))?;
            let amount = Amount::from(blinded_message.amount);

            let key_pair = {
                let keyset = keyset_cache_read_lock
                    .get(&keyset_id)
                    .ok_or(Status::not_found(Error::KeysetNotFound(keyset_id)))?;
                keyset
                    .get(&amount)
                    .ok_or(Status::not_found(Error::AmountNotFound(amount, keyset_id)))?
            };

            let blind_secret = PublicKey::from_slice(&blinded_message.blinded_secret)
                .map_err(|_| Status::invalid_argument(Error::BadSecret))?;

            let c = sign_message(&key_pair.secret_key, &blind_secret)
                .map_err(|e| Status::internal(Error::Dhke(e)))?;

            signatures.push(c.to_bytes().to_vec());
        }

        Ok(Response::new(SignBlindedMessagesResponse { signatures }))
    }

    async fn verify_proofs(
        &self,
        verify_proofs_request: Request<VerifyProofsRequest>,
    ) -> Result<Response<VerifyProofsResponse>, Status> {
        for proof in &verify_proofs_request.get_ref().proofs {
            let keyset_id = KeysetId::from_bytes(&proof.keyset_id)
                .map_err(|_| Status::invalid_argument(Error::BadKeysetId))?;
            let amount = Amount::from(proof.amount);

            let secret_key = {
                let keyset_cache_read_lock = self
                    .keyset_cache
                    .0
                    .read()
                    .map_err(|_| Status::internal(Error::LockPoisoned))?;

                let keyset = keyset_cache_read_lock
                    .get(&keyset_id)
                    .ok_or(Status::not_found(Error::KeysetNotFound(keyset_id)))?;
                keyset
                    .get(&amount)
                    .ok_or(Status::not_found(Error::AmountNotFound(amount, keyset_id)))?
                    .secret_key
                    .clone()
            };

            let c = PublicKey::from_slice(&proof.unblind_signature)
                .map_err(|_| Status::invalid_argument(Error::BadC))?;

            if !verify_message(&secret_key, c, proof.secret.as_bytes())
                .map_err(|e| Status::internal(Error::Dhke(e)))?
            {
                return Ok(Response::new(VerifyProofsResponse { is_valid: false }));
            };
        }

        Ok(Response::new(VerifyProofsResponse { is_valid: true }))
    }

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
    #[cfg(debug_assertions)]
    dotenvy::from_filename("signer.env")?;

    let socket_addr = {
        let socket_ip_env_var: String =
            std::env::var(SOCKET_IP_ENV_VAR).expect("env var `SOCKET_IP` should be set");
        let socket_port_env_var: String =
            std::env::var(SOCKET_PORT_ENV_VAR).expect("env var `SOCKET_PORT` should be set");
        format!("{}:{}", socket_ip_env_var, socket_port_env_var).parse()?
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

    let svc = SignerServer::new(signer_logic);

    println!("listening to new request on {}", socket_addr);

    Server::builder()
        .add_service(svc)
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
