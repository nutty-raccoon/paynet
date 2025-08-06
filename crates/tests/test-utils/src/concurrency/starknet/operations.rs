use std::{collections::HashSet, time::Duration};

use futures::future::join_all;
use node_client::{
    AcknowledgeRequest, BlindedMessage, GetKeysRequest, MeltQuoteRequest, MeltRequest,
    MeltResponse, MintQuoteRequest, MintQuoteResponse, MintRequest, MintResponse, NodeClient,
    Proof, SwapRequest, SwapResponse, hash_mint_request,
};
use nuts::{
    Amount,
    dhke::{blind_message, unblind_message},
    nut00::secret::Secret,
    nut01::PublicKey,
};
use primitive_types::U256;
use starknet_types::{DepositPayload, STARKNET_STR, Unit, constants::ON_CHAIN_CONSTANTS};
use starknet_types_core::felt::Felt;
use tonic::transport::Channel;

use crate::{
    common::{
        error::{Error, Result},
        utils::{EnvVariables, starknet::pay_invoices},
    },
    concurrency::starknet::utils::{
        get_active_keyset, make_melt, make_mint, make_swap, mint_quote_and_deposit_and_wait,
        wait_transac,
    },
};

// Concurrency tests for mint, swap, and melt operations.

// Verifies double-spending protection by attempting to reuse a single quote across multiple concurrent mint operations
pub async fn mint_same_quote(node_client: NodeClient<Channel>, env: EnvVariables) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

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

    let mut mints = Vec::new();
    for req in mints_requests {
        mints.push(make_mint(req, node_client.clone()));
    }

    let res = join_all(mints).await;

    let ok_vec: Vec<&MintResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Mint,
        ));
    }

    Ok(())
}

/// Tests output collision detection by using identical blinded secrets across multiple concurrent mint operations
pub async fn mint_same_output(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
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

    let on_chain_constants = ON_CHAIN_CONSTANTS.get(env.chain_id.as_str()).unwrap();
    // Edit the allow call so that one call is enough to cover all invoices
    // Then we only push the payment_invoice call. This reduce by half the number of calls.
    // It is important because something break in DNA when there is too many calls, or events
    // in a single transaction.
    // That is the reason why we use `50` as the size of a batch, 100 was breaking it
    let deposit_payload: DepositPayload =
        serde_json::from_str(&mint_quote_response_iterator.next().unwrap().request)?;
    let mut c =
        deposit_payload.to_starknet_calls(on_chain_constants.invoice_payment_contract_address);
    c[0].calldata[1] *= Felt::from(100);
    calls.push(c[0].clone());
    calls.push(c[1].clone());
    let mut i = 0;
    for quote in mint_quote_response_iterator {
        let deposit_payload: DepositPayload = serde_json::from_str(&quote.request)?;
        let c =
            deposit_payload.to_starknet_calls(on_chain_constants.invoice_payment_contract_address);
        calls.push(c[1].clone());
        i += 1;

        // Every 50 quote, we send a transaction
        if i == 50 {
            pay_invoices(calls.clone(), env.clone()).await?;
            i = 0;
            calls.clear();
        }
    }
    // Won't be called with current values but protect us agains regression
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
    let mut mints = Vec::new();
    for req in mints_requests {
        mints.push(make_mint(req, node_client.clone()));
    }

    let res = join_all(mints).await;

    let ok_vec: Vec<&MintResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Mint,
        ));
    }

    Ok(())
}

/// Ensures swap atomicity by attempting to generate identical output tokens from different inputs concurrently
pub async fn swap_same_output(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    let swap_amount = 128u64;
    let n_concurent = 64;
    let total_amount_to_mint = Amount::from(swap_amount * n_concurent);

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
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
            .find(|key| key.amount == swap_amount)
            .unwrap()
            .pubkey,
    )
    .map_err(|e| Error::Other(e.into()))?;
    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), total_amount_to_mint)
            .await?;

    let mut blind_messages = Vec::with_capacity(n_concurent as usize);
    let mut rs = Vec::with_capacity(n_concurent as usize);
    let mut secrets = Vec::with_capacity(n_concurent as usize);
    for _ in 0..n_concurent {
        let secret = Secret::generate();
        let (blinded_secret, r) =
            blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
        blind_messages.push(BlindedMessage {
            amount: swap_amount,
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        });
        rs.push(r);
        secrets.push(secret);
    }

    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: original_mint_quote_response.clone().quote,
        outputs: blind_messages,
    };
    let mint_response = make_mint(mint_request, node_client.clone()).await?;
    let proofs: Vec<_> = mint_response
        .signatures
        .into_iter()
        .enumerate()
        .map(|(i, s)| Proof {
            amount: Amount::from(swap_amount).into(),
            keyset_id: active_keyset.id.clone(),
            secret: secrets[i].to_string(),
            unblind_signature: unblind_message(
                &PublicKey::from_slice(&s.blind_signature).unwrap(),
                &rs[i],
                &node_pubkey_for_amount,
            )
            .unwrap()
            .to_bytes()
            .to_vec(),
        })
        .collect();

    let secret = Secret::generate();
    let (blinded_secret, _r) =
        blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;
    let blinded_message = BlindedMessage {
        amount: swap_amount,
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let mut multi_swap = Vec::new();
    for proof in proofs {
        let swap_request = SwapRequest {
            inputs: vec![proof],
            outputs: vec![blinded_message.clone()],
        };
        multi_swap.push(make_swap(node_client.clone(), swap_request));
    }
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Swap,
        ));
    }
    Ok(())
}

/// Validates double-spending prevention by reusing the same proof across multiple concurrent swap operations
pub async fn swap_same_input(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

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
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Swap,
        ));
    }
    Ok(())
}

/// Tests melt operation integrity by attempting to spend the same proof multiple times concurrently
pub async fn melt_same_input(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    // MINTING

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

    let on_chain_constants = ON_CHAIN_CONSTANTS.get(env.chain_id.as_str()).unwrap();
    let deposit_payload: DepositPayload =
        serde_json::from_str(&original_mint_quote_response.request)?;
    pay_invoices(
        deposit_payload
            .to_starknet_calls(on_chain_constants.invoice_payment_contract_address)
            .to_vec(),
        env,
    )
    .await?;
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

    let mut melt_quote_ids: Vec<String> = Vec::new();
    // Build a set of recipient
    let mut payees: HashSet<Felt> = HashSet::new();
    for i in 0..100 {
        // we start at 0x02 because the first two address is not valid
        let addr = "0x02".to_string() + &i.to_string();
        payees.insert(Felt::from_hex(&addr).map_err(|e| Error::Other(e.into()))?);
    }

    let method = STARKNET_STR.to_string();
    let asset = starknet_types::Asset::Strk;
    let on_chain_amount = U256::from(32).checked_mul(asset.scale_factor()).unwrap() / 1000;
    for payee in payees.iter() {
        let melt_quote_response = node_client
            .melt_quote(MeltQuoteRequest {
                method: method.clone(),
                unit: Unit::MilliStrk.to_string(),
                request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                    payee: *payee,
                    asset,
                    amount: on_chain_amount.into(),
                })?,
            })
            .await?
            .into_inner();
        assert_eq!(melt_quote_response.amount, proof.amount);
        melt_quote_ids.push(melt_quote_response.quote);
    }

    let mut multi_melt = Vec::new();
    for melt_quote_id in melt_quote_ids.iter() {
        let melt_request = MeltRequest {
            method: method.clone(),
            quote: melt_quote_id.clone(),
            inputs: vec![proof.clone()],
        };
        multi_melt.push(make_melt(node_client.clone(), melt_request));
    }
    let res = join_all(multi_melt).await;
    let ok_vec: Vec<(usize, &MeltResponse)> = res
        .iter()
        .enumerate()
        .filter_map(|(i, res)| res.as_ref().ok().map(|r| (i, r)))
        .collect();
    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Melt,
        ));
    }
    println!("succes: {}", ok_vec.len());

    let (quote_index, _) = ok_vec.first().unwrap();

    // Wait for payment to go through
    loop {
        let response = node_client
            .melt_quote_state(node_client::MeltQuoteStateRequest {
                method: method.clone(),
                quote: melt_quote_ids[*quote_index].clone(),
            })
            .await?
            .into_inner();

        if response.state == node_client::MeltQuoteState::MlqsPaid as i32 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

// Tests melt operation integrity by attempting to spend the same quote multiple times concurrently

pub async fn melt_same_quote(
    mut node_client: NodeClient<Channel>,

    env: EnvVariables,
) -> Result<()> {
    let melt_amount = 128u64;
    let n_concurent = 64;
    let total_amount_to_mint = Amount::from(melt_amount * n_concurent);

    // MINTING
    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;

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
            .find(|key| key.amount == melt_amount)
            .unwrap()
            .pubkey,
    )
    .map_err(|e| Error::Other(e.into()))?;

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), total_amount_to_mint)
            .await?;

    let mut blind_messages = Vec::with_capacity(n_concurent as usize);
    let mut rs = Vec::with_capacity(n_concurent as usize);
    let mut secrets = Vec::with_capacity(n_concurent as usize);

    for _ in 0..n_concurent {
        let secret = Secret::generate();

        let (blinded_secret, r) =
            blind_message(secret.as_bytes(), None).map_err(|e| Error::Other(e.into()))?;

        blind_messages.push(BlindedMessage {
            amount: melt_amount,
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        });

        rs.push(r);
        secrets.push(secret);
    }

    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: original_mint_quote_response.clone().quote,
        outputs: blind_messages,
    };

    let mint_response = make_mint(mint_request, node_client.clone()).await?;

    let proofs: Vec<_> = mint_response
        .signatures
        .into_iter()
        .enumerate()
        .map(|(i, s)| Proof {
            amount: Amount::from(melt_amount).into(),

            keyset_id: active_keyset.id.clone(),

            secret: secrets[i].to_string(),

            unblind_signature: unblind_message(
                &PublicKey::from_slice(&s.blind_signature).unwrap(),
                &rs[i],
                &node_pubkey_for_amount,
            )
            .unwrap()
            .to_bytes()
            .to_vec(),
        })
        .collect();

    // MELT
    let payee =
        Felt::from_hex("0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")
            .map_err(|e| Error::Other(e.into()))?;

    let method = STARKNET_STR.to_string();

    let asset = starknet_types::Asset::Strk;

    let on_chain_amount = U256::from(128).checked_mul(asset.scale_factor()).unwrap() / 1000;

    let melt_quote_response = node_client
        .melt_quote(MeltQuoteRequest {
            method: method.clone(),
            unit: Unit::MilliStrk.to_string(),
            request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                payee,
                asset,
                amount: on_chain_amount.into(),
            })?,
        })
        .await?
        .into_inner();

    let melt_quote_id = melt_quote_response.quote;
    let mut melt_requests = Vec::new();

    for proof in proofs {
        let melt_request = MeltRequest {
            method: method.clone(),
            quote: melt_quote_id.clone(),
            inputs: vec![proof],
        };

        melt_requests.push(make_melt(node_client.clone(), melt_request));
    }

    let res = join_all(melt_requests).await;
    let ok_vec: Vec<&MeltResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();

    println!("success: {}", ok_vec.len());

    if ok_vec.len() != 1 {
        return Err(Error::Concurrence(
            crate::common::error::ConcurrencyError::Melt,
        ));
    }

    // Wait for payment to go through
    loop {
        let response = node_client
            .melt_quote_state(node_client::MeltQuoteStateRequest {
                method: method.clone(),
                quote: melt_quote_id.clone(),
            })
            .await?
            .into_inner();

        if response.state == node_client::MeltQuoteState::MlqsPaid as i32 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
