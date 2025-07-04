use futures::future::join_all;
use node::{
    AcknowledgeRequest, BlindedMessage, GetKeysRequest, MeltRequest, MeltResponse,
    MintQuoteRequest, MintQuoteResponse, MintRequest, MintResponse, NodeClient, Proof, SwapRequest,
    SwapResponse, hash_mint_request,
};
use nuts::{
    Amount,
    dhke::{blind_message, unblind_message},
    nut00::secret::Secret,
    nut01::PublicKey,
};
use starknet_types::Unit;
use starknet_types_core::felt::Felt;
use tonic::transport::Channel;

use crate::{
    concurrency::utils::{
        get_active_keyset, make_melt, make_mint, make_swap, mint_and_wait, wait_transac,
    },
    env_variables::EnvVariables,
    errors::{Error, Result},
    utils::pay_invoices,
};

/// Concurrency tests for mint, swap, and melt operations.

/// Verifies double-spending protection by attempting to reuse a single quote across multiple concurrent mint operations
pub async fn mint_same_quote(node_client: NodeClient<Channel>, env: EnvVariables) -> Result<()> {
    println!("  * running mint concurency test");
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_and_wait(&mut node_client.clone(), &env.clone(), amount).await?;

    let mut mints_requests: Vec<MintRequest> = Vec::new();
    for _ in 0..100 {
        let active_keyset =
            get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
        let secret = Secret::generate();
        let (blinded_secret, _r) =
            blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
        let mint_request = MintRequest {
            method: "starknet".to_string(),
            quote: original_mint_quote_response.clone().quote,
            outputs: vec![BlindedMessage {
                amount: amount.into(),
                keyset_id: active_keyset.id.clone(),
                blinded_secret: blinded_secret.to_bytes().to_vec(),
            }],
        };
        mints_requests.push(mint_request);
    }

    println!("minting all");
    let res = join_all(
        mints_requests
            .iter()
            .map(|req| make_mint(req.clone(), node_client.clone())),
    )
    .await;

    let ok_vec: Vec<&MintResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{} success", ok_vec.len());
    assert_eq!(ok_vec.len(), 1);

    Ok(())
}

/// Tests output collision detection by using identical blinded secrets across multiple concurrent mint operations
pub async fn mint_same_output(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    println!("  * running mint_same_output test");
    let amount = Amount::from_i64_repr(8);

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

    let mut calls = Vec::with_capacity(51);
    let mut mint_quote_response_iterator = mints_quote_response.iter();

    // Edit the allow call so that one call is enough to cover all invoices
    // Then we only push the payment_invoice call. This reduce by half the number of calls.
    // It is important because something break in DNA when there is too many calls, or events
    // in a single transaction.
    // That is the reason why we use `50` as the size of a batch, 100 was breaking it
    let mut c: [starknet_types::Call; 2] =
        serde_json::from_str(&mint_quote_response_iterator.next().unwrap().request)?;
    c[0].calldata[1] *= Felt::from(100);
    calls.push(c[0].clone());
    calls.push(c[1].clone());
    let mut i = 0;
    for quote in mint_quote_response_iterator {
        let c: [starknet_types::Call; 2] = serde_json::from_str(&quote.request)?;
        calls.push(c[1].clone());
        i += 1;

        // Every 50 quote, we send a transaction
        if i == 50 {
            i = 0;
            pay_invoices(calls.clone(), env.clone()).await?;
            calls.clear();
        }
    }
    // Won't be called by protect us agains regression
    // if we change the number of concurrent calls in the future
    if !calls.is_empty() {
        pay_invoices(calls, env.clone()).await?;
    }

    for quote in &mints_quote_response {
        wait_transac(node_client.clone(), quote).await?;
    }

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
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

    println!("minting all");
    let res = join_all(
        mints_requests
            .iter()
            .map(|req| make_mint(req.clone(), node_client.clone())),
    )
    .await;

    let ok_vec: Vec<&MintResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{} success", ok_vec.len());
    assert_eq!(ok_vec.len(), 1);

    Ok(())
}

/// Ensures swap atomicity by attempting to generate identical output tokens from different inputs concurrently
pub async fn swap_same_output(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    println!("  * running swap_same_output test");
    let amount = Amount::from_i64_repr(128);

    let original_mint_quote_response =
        mint_and_wait(&mut node_client.clone(), &env.clone(), amount).await?;

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;

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

    let mut multi_proof = Vec::new();
    for _ in 0..100 {
        let proof = Proof {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            secret: secret.to_string(),
            unblind_signature: unblinded_signature.to_bytes().to_vec(),
        };
        multi_proof.push(proof);
    }

    let secret = Secret::generate();
    let (blinded_secret, _r) =
        blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
    let blinded_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let mut multi_swap = Vec::new();
    for proof in multi_proof {
        let swap_request = SwapRequest {
            inputs: vec![proof],
            outputs: vec![blinded_message.clone()],
        };
        multi_swap.push(make_swap(node_client.clone(), swap_request));
    }
    println!("swapping all");
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{} success", ok_vec.len());
    assert_eq!(ok_vec.len(), 1);
    Ok(())
}

/// Validates double-spending prevention by reusing the same proof across multiple concurrent swap operations
pub async fn swap_same_input(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    println!("  * running swap_same_input test");
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_and_wait(&mut node_client.clone(), &env.clone(), amount).await?;

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
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

    let mut multi_swap = Vec::new();
    for _ in 0..100 {
        let secret = Secret::generate();
        let (blinded_secret, _r) =
            blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
        let blind_message = BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        };
        let swap_request = SwapRequest {
            inputs: vec![proof.clone()],
            outputs: vec![blind_message],
        };
        multi_swap.push(make_swap(node_client.clone(), swap_request.clone()))
    }
    println!("swapping all");
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{} success", ok_vec.len());
    assert_eq!(ok_vec.len(), 1);
    Ok(())
}

/// Tests melt operation integrity by attempting to spend the same proof multiple times concurrently
pub async fn melt_same_input(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    println!("  * running melt_same_input test");
    let payee =
        Felt::from_hex(&"0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")
            .map_err(|e| Error::Other(e.into()))?;

    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_and_wait(&mut node_client.clone(), &env.clone(), amount).await?;

    let calls: [starknet_types::Call; 2] =
        serde_json::from_str(&original_mint_quote_response.request)?;
    pay_invoices(calls.to_vec(), env).await?;

    wait_transac(node_client.clone(), &original_mint_quote_response).await?;

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
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

    let melt_request = MeltRequest {
        method: "starknet".to_string(),
        unit: Unit::MilliStrk.to_string(),
        request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
            payee,
            asset: starknet_types::Asset::Strk,
        })?,
        inputs: vec![proof],
    };

    let mut multi_melt = Vec::new();
    for _ in 0..100 {
        multi_melt.push(make_melt(node_client.clone(), melt_request.clone()));
    }
    println!("melting all");
    let res = join_all(multi_melt).await;
    let ok_vec: Vec<&MeltResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    println!("{} success", ok_vec.len());
    assert_eq!(ok_vec.len(), 1);
    Ok(())
}
