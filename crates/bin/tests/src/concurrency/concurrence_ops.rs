use futures::future::join_all;
use node::{
    AcknowledgeRequest, BlindedMessage, GetKeysRequest, GetKeysetsRequest, MintQuoteRequest,
    MintQuoteResponse, MintQuoteState, MintRequest, MintResponse, NodeClient, Proof,
    QuoteStateRequest, SwapRequest, SwapResponse, hash_mint_request, hash_swap_request,
};
use nuts::{
    Amount,
    dhke::{blind_message, unblind_message},
    nut00::secret::Secret,
    nut01::PublicKey,
};
use starknet_types::Unit;
use tonic::transport::Channel;

use crate::{
    env_variables::EnvVariables,
    errors::{Error, Result},
    utils::pay_invoice,
};

pub async fn blind_message_concurrence(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    let amount = Amount::from_i64_repr(10);

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let mut mints_quote_response: Vec<MintQuoteResponse> = Vec::new();
    for _ in 0..100 {
        mints_quote_response.push({
            node_client
                .mint_quote(mint_quote_request.clone())
                .await?
                .into_inner()
        })
    }

    for quote in &mints_quote_response {
        pay_invoice(quote.request.clone(), env.clone()).await?;
    }

    for quote in &mints_quote_response {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let response = node_client
                .mint_quote_state(QuoteStateRequest {
                    method: "starknet".to_string(),
                    quote: quote.quote.clone(),
                })
                .await;

            match response {
                Ok(response) => {
                    let response = response.into_inner();
                    let state = MintQuoteState::try_from(response.state)
                        .map_err(|e| Error::Other(e.into()))?;
                    if state == MintQuoteState::MnqsPaid {
                        break;
                    }
                }
                Err(e) => {
                    println!("{e}")
                }
            }
        }
    }

    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();
    let secret = Secret::generate();
    let (blinded_secret, _r) =
        blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
    let mut mints_requests: Vec<MintRequest> = Vec::new();
    for quote in &mints_quote_response {
        mints_requests.push(MintRequest {
            method: "starknet".to_string(),
            quote: quote.quote.clone(),
            outputs: vec![BlindedMessage {
                amount: amount.into(),
                keyset_id: active_keyset.id.clone(),
                blinded_secret: blinded_secret.to_bytes().to_vec(),
            }],
        });
    }
    let mut mints = Vec::new();
    for req in mints_requests {
        mints.push(make_mint(req, node_client.clone()));
    }

    let res = join_all(mints).await;

    let ok_vec: Vec<&MintResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{}", ok_vec.len());

    Ok(())
}

async fn make_mint(req: MintRequest, mut node_client: NodeClient<Channel>) -> Result<MintResponse> {
    let mint_response = node_client.mint(req.clone()).await?.into_inner();
    let request_hash = hash_mint_request(&req);
    node_client
        .acknowledge(AcknowledgeRequest {
            path: "mint".to_string(),
            request_hash,
        })
        .await?;
    Ok(mint_response)
}

pub async fn swap_concurrence(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let original_mint_quote_response = node_client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

    pay_invoice(original_mint_quote_response.request, env).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let response = node_client
            .mint_quote_state(QuoteStateRequest {
                method: "starknet".to_string(),
                quote: original_mint_quote_response.quote.clone(),
            })
            .await;

        match response {
            Ok(response) => {
                let response = response.into_inner();
                let state =
                    MintQuoteState::try_from(response.state).map_err(|e| Error::Other(e.into()))?;
                if state == MintQuoteState::MnqsPaid {
                    break;
                }
            }
            Err(e) => {
                println!("{e}")
            }
        }
    }

    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();
    let secret = Secret::generate();
    let (blinded_secret, r) =
        blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: original_mint_quote_response.quote,
        outputs: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        }],
    };

    let original_mint_response = node_client.mint(mint_request.clone()).await?.into_inner();
    let request_hash = hash_mint_request(&mint_request);
    node_client
        .acknowledge(AcknowledgeRequest {
            path: "mint".to_string(),
            request_hash,
        })
        .await?;

    let node_pubkey_for_amount = PublicKey::from_hex(
        &node_client
            .keys(GetKeysRequest {
                keyset_id: Some(active_keyset.id.clone()),
            })
            .await?
            .into_inner()
            .keysets
            .first()
            .unwrap()
            .keys
            .iter()
            .find(|key| Amount::from(key.amount) == amount)
            .unwrap()
            .pubkey,
    )
    .map_err(|e| Error::Other(e.into()))?;
    let blind_signature = PublicKey::from_slice(
        &original_mint_response
            .signatures
            .first()
            .unwrap()
            .blind_signature,
    )
    .unwrap();
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)
        .map_err(|e| Error::Other(e.into()))?;
    let proof = Proof {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        secret: secret.to_string(),
        unblind_signature: unblinded_signature.to_bytes().to_vec(),
    };

    let secret = Secret::generate();
    let (blinded_secret, _r) =
        blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
    let blind_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let swap_request = SwapRequest {
        inputs: vec![proof],
        outputs: vec![blind_message],
    };
    let mut multi_swap = Vec::new();
    for _ in 0..100 {
        multi_swap.push(make_swap(node_client.clone(), swap_request.clone()))
    }
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    assert_eq!(ok_vec.len(), 1);
    Ok(())
}

async fn make_swap(
    mut node_client: NodeClient<Channel>,
    swap_request: SwapRequest,
) -> Result<SwapResponse> {
    let original_swap_response = node_client.swap(swap_request.clone()).await?.into_inner();
    let request_hash = hash_swap_request(&swap_request);
    node_client
        .acknowledge(AcknowledgeRequest {
            path: "swap".to_string(),
            request_hash,
        })
        .await?;
    Ok(original_swap_response)
}
