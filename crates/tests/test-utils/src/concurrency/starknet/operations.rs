use anyhow::Result;
use cashu_client::{CashuClient, ClientMeltQuoteRequest, ClientMintQuoteRequest};
use std::{collections::HashSet, time::Duration};

use futures::future::join_all;
use nuts::{
    Amount,
    dhke::{blind_message, unblind_message},
    nut00::secret::Secret,
    nut02::KeysetId,
    nut19::hash_mint_request,
};
use primitive_types::U256;
use starknet_types::{DepositPayload, STARKNET_STR, Unit, constants::ON_CHAIN_CONSTANTS};
use starknet_types_core::felt::Felt;

use crate::{
    common::utils::{EnvVariables, starknet::pay_invoices},
    concurrency::starknet::utils::{
        get_active_keyset, make_melt, make_mint, make_swap, mint_quote_and_deposit_and_wait,
        wait_transac,
    },
};

// Concurrency tests for mint, swap, and melt operations.

// Verifies double-spending protection by attempting to reuse a single quote across multiple concurrent mint operations
pub async fn mint_same_quote(node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

    let mut mints_requests: Vec<nuts::nut04::MintRequest<String>> = Vec::new();
    for _ in 0..100 {
        let active_keyset =
            get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
        let secret = Secret::generate();
        let (blinded_secret, _r) = blind_message(secret.as_bytes(), None)?;
        let mint_request = nuts::nut04::MintRequest {
            quote: original_mint_quote_response.quote.clone(),
            outputs: vec![nuts::nut00::BlindedMessage {
                amount,
                keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
                blinded_secret,
            }],
        };
        mints_requests.push(mint_request);
    }

    let mut mints = Vec::new();
    for req in mints_requests {
        mints.push(make_mint(req, node_client.clone()));
    }

    let res = join_all(mints).await;

    let ok_vec: Vec<&nuts::nut04::MintResponse> =
        res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Mint)?;
    }

    Ok(())
}

/// Tests output collision detection by using identical blinded secrets across multiple concurrent mint operations
pub async fn mint_same_output(mut node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let amount = Amount::from_i64_repr(8);

    let mint_quote_request = ClientMintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let mut mints_quote_response: Vec<nuts::nut04::MintQuoteResponse<String>> = Vec::new();
    for _ in 0..100 {
        mints_quote_response.push(node_client.mint_quote(mint_quote_request.clone()).await?)
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
    let mut c = deposit_payload
        .call_data
        .to_starknet_calls(on_chain_constants.invoice_payment_contract_address);
    c[0].calldata[1] *= Felt::from(100);
    calls.push(c[0].clone());
    calls.push(c[1].clone());
    let mut i = 0;
    for quote in mint_quote_response_iterator {
        let deposit_payload: DepositPayload = serde_json::from_str(&quote.request)?;
        let c = deposit_payload
            .call_data
            .to_starknet_calls(on_chain_constants.invoice_payment_contract_address);
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
    let (blinded_secret, _r) = blind_message(secret.as_bytes(), None)?;
    let mut mints_requests: Vec<nuts::nut04::MintRequest<String>> = Vec::new();
    for quote in &mints_quote_response {
        mints_requests.push(nuts::nut04::MintRequest {
            quote: quote.quote.clone(),
            outputs: vec![nuts::nut00::BlindedMessage {
                amount,
                keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
                blinded_secret,
            }],
        });
    }
    let mut mints = Vec::new();
    for req in mints_requests {
        mints.push(make_mint(req, node_client.clone()));
    }

    let res = join_all(mints).await;

    let ok_vec: Vec<&nuts::nut04::MintResponse> =
        res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Mint)?;
    }

    Ok(())
}

/// Ensures swap atomicity by attempting to generate identical output tokens from different inputs concurrently
pub async fn swap_same_output(mut node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let swap_amount = 128u64;
    let n_concurent = 64;
    let total_amount_to_mint = Amount::from(swap_amount * n_concurent);

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
    let node_pubkey_for_amount = node_client
        .keys(Some(
            KeysetId::from_bytes(&active_keyset.id.clone()).unwrap(),
        ))
        .await?
        .keysets
        .first()
        .unwrap()
        .keys
        .iter()
        .find(|key| key.amount == Amount::from(swap_amount))
        .unwrap()
        .publickey;
    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), total_amount_to_mint)
            .await?;

    let mut blind_messages = Vec::with_capacity(n_concurent as usize);
    let mut rs = Vec::with_capacity(n_concurent as usize);
    let mut secrets = Vec::with_capacity(n_concurent as usize);
    for _ in 0..n_concurent {
        let secret = Secret::generate();
        let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
        blind_messages.push(nuts::nut00::BlindedMessage {
            amount: Amount::from(swap_amount),
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        });
        rs.push(r);
        secrets.push(secret);
    }

    let mint_request = nuts::nut04::MintRequest {
        quote: original_mint_quote_response.quote.clone(),
        outputs: blind_messages,
    };

    let mint_response = make_mint(mint_request, node_client.clone()).await?;
    let proofs: Vec<nuts::nut00::Proof> = mint_response
        .signatures
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            Ok(nuts::nut00::Proof {
                amount: Amount::from(swap_amount),
                keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
                secret: secrets[i].clone(),
                c: unblind_message(&s.c, &rs[i], &node_pubkey_for_amount)?,
            })
        })
        .collect::<Result<Vec<nuts::nut00::Proof>>>()?;

    let secret = Secret::generate();
    let (blinded_secret, _r) = blind_message(secret.as_bytes(), None)?;
    let blinded_message = nuts::nut00::BlindedMessage {
        amount: Amount::from(swap_amount),
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        blinded_secret,
    };

    let mut multi_swap = Vec::new();
    for proof in proofs {
        let swap_request = nuts::nut03::SwapRequest {
            inputs: vec![proof],
            outputs: vec![blinded_message.clone()],
        };
        multi_swap.push(make_swap(node_client.clone(), swap_request));
    }
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&nuts::nut03::SwapResponse> =
        res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Swap)?;
    }
    Ok(())
}

/// Validates double-spending prevention by reusing the same proof across multiple concurrent swap operations
pub async fn swap_same_input(mut node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;

    let mint_request = nuts::nut04::MintRequest {
        quote: original_mint_quote_response.quote,
        outputs: vec![nuts::nut00::BlindedMessage {
            amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        }],
    };

    let original_mint_response = node_client
        .mint(mint_request.clone(), "starknet".to_string())
        .await?;
    let request_hash = hash_mint_request(&mint_request);
    node_client
        .acknowledge("mint".to_string(), request_hash)
        .await?;

    let node_pubkey_for_amount = node_client
        .keys(Some(
            KeysetId::from_bytes(&active_keyset.id.clone()).unwrap(),
        ))
        .await?
        .keysets
        .first()
        .unwrap()
        .keys
        .iter()
        .find(|key| key.amount == amount)
        .unwrap()
        .publickey;
    let blind_signature = original_mint_response.signatures.first().unwrap().c;
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = nuts::nut00::Proof {
        amount,
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        secret,
        c: unblinded_signature,
    };

    let mut multi_swap = Vec::new();
    for _ in 0..100 {
        let secret = Secret::generate();
        let (blinded_secret, _r) = blind_message(secret.as_bytes(), None)?;
        let blind_message = nuts::nut00::BlindedMessage {
            amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        };
        let swap_request = nuts::nut03::SwapRequest {
            inputs: vec![proof.clone()],
            outputs: vec![blind_message],
        };
        multi_swap.push(make_swap(node_client.clone(), swap_request.clone()))
    }
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&nuts::nut03::SwapResponse> =
        res.iter().filter_map(|res| res.as_ref().ok()).collect();
    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Swap)?;
    }
    Ok(())
}

/// Tests melt operation integrity by attempting to spend the same proof multiple times concurrently
pub async fn melt_same_input(mut node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let amount = Amount::from_i64_repr(32);

    // MINTING

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), amount).await?;

    let on_chain_constants = ON_CHAIN_CONSTANTS.get(env.chain_id.as_str()).unwrap();
    let deposit_payload: DepositPayload =
        serde_json::from_str(&original_mint_quote_response.request)?;
    pay_invoices(
        deposit_payload
            .call_data
            .to_starknet_calls(on_chain_constants.invoice_payment_contract_address)
            .to_vec(),
        env,
    )
    .await?;
    wait_transac(node_client.clone(), &original_mint_quote_response).await?;

    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;
    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;

    let mint_request = nuts::nut04::MintRequest {
        quote: original_mint_quote_response.quote,
        outputs: vec![nuts::nut00::BlindedMessage {
            amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        }],
    };

    let original_mint_response = node_client
        .mint(mint_request.clone(), "starknet".to_string())
        .await?;
    let request_hash = hash_mint_request(&mint_request);
    node_client
        .acknowledge("mint".to_string(), request_hash)
        .await?;

    let node_pubkey_for_amount = node_client
        .keys(Some(
            KeysetId::from_bytes(&active_keyset.id.clone()).unwrap(),
        ))
        .await?
        .keysets
        .first()
        .unwrap()
        .keys
        .iter()
        .find(|key| key.amount == amount)
        .unwrap()
        .publickey;
    let blind_signature = original_mint_response.signatures.first().unwrap().c;
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = nuts::nut00::Proof {
        amount,
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        secret,
        c: unblinded_signature,
    };

    let mut melt_quote_ids: Vec<String> = Vec::new();
    // Build a set of recipient
    let mut payees: HashSet<Felt> = HashSet::new();
    for i in 0..100 {
        // we start at 0x02 because the first two address is not valid
        let addr = "0x02".to_string() + &i.to_string();
        payees.insert(Felt::from_hex(&addr)?);
    }

    let method = STARKNET_STR.to_string();
    let asset = starknet_types::Asset::Strk;
    let on_chain_amount = U256::from(32).checked_mul(asset.scale_factor()).unwrap() / 1000;
    for payee in payees.iter() {
        let melt_quote_response = node_client
            .melt_quote(ClientMeltQuoteRequest {
                method: "starknet".to_string(),
                unit: Unit::MilliStrk.to_string(),
                request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                    payee: *payee,
                    asset,
                    amount: on_chain_amount.into(),
                })?,
            })
            .await?;
        assert_eq!(melt_quote_response.amount, proof.amount);
        melt_quote_ids.push(melt_quote_response.quote);
    }

    let mut multi_melt = Vec::new();
    for melt_quote_id in melt_quote_ids.iter() {
        let melt_request = nuts::nut05::MeltRequest {
            quote: melt_quote_id.clone(),
            inputs: vec![proof.clone()],
        };
        multi_melt.push(make_melt(node_client.clone(), melt_request));
    }
    let res = join_all(multi_melt).await;
    let ok_vec: Vec<(usize, &nuts::nut05::MeltResponse)> = res
        .iter()
        .enumerate()
        .filter_map(|(i, res)| res.as_ref().ok().map(|r| (i, r)))
        .collect();
    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Melt)?;
    }
    println!("succes: {}", ok_vec.len());

    let (quote_index, _) = ok_vec.first().unwrap();

    // Wait for payment to go through
    loop {
        let response = node_client
            .melt_quote_state(method.clone(), melt_quote_ids[*quote_index].clone())
            .await?;
        if response.state == nuts::nut05::MeltQuoteState::Paid {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

// Tests melt operation integrity by attempting to spend the same quote multiple times concurrently

pub async fn melt_same_quote(mut node_client: impl CashuClient, env: EnvVariables) -> Result<()> {
    let melt_amount = 128u64;
    let n_concurent = 64;
    let total_amount_to_mint = Amount::from(melt_amount * n_concurent);

    // MINTING
    let active_keyset =
        get_active_keyset(&mut node_client.clone(), Unit::MilliStrk.as_str()).await?;

    let node_pubkey_for_amount = node_client
        .keys(Some(
            KeysetId::from_bytes(&active_keyset.id.clone()).unwrap(),
        ))
        .await?
        .keysets
        .first()
        .unwrap()
        .keys
        .iter()
        .find(|key| key.amount == Amount::from(melt_amount))
        .unwrap()
        .publickey;

    let original_mint_quote_response =
        mint_quote_and_deposit_and_wait(node_client.clone(), env.clone(), total_amount_to_mint)
            .await?;

    let mut blind_messages = Vec::with_capacity(n_concurent as usize);
    let mut rs = Vec::with_capacity(n_concurent as usize);
    let mut secrets = Vec::with_capacity(n_concurent as usize);

    for _ in 0..n_concurent {
        let secret = Secret::generate();

        let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;

        blind_messages.push(nuts::nut00::BlindedMessage {
            amount: Amount::from(melt_amount),
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        });

        rs.push(r);
        secrets.push(secret);
    }

    let mint_request = nuts::nut04::MintRequest {
        quote: original_mint_quote_response.quote.clone(),
        outputs: blind_messages,
    };

    let mint_response = make_mint(mint_request, node_client.clone()).await?;

    let proofs: Vec<nuts::nut00::Proof> = mint_response
        .signatures
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            Ok(nuts::nut00::Proof {
                amount: Amount::from(melt_amount),
                keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
                secret: secrets[i].clone(),
                c: unblind_message(&s.c, &rs[i], &node_pubkey_for_amount)?,
            })
        })
        .collect::<Result<Vec<nuts::nut00::Proof>>>()?;

    // MELT
    let payee =
        Felt::from_hex("0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")?;

    let method = STARKNET_STR.to_string();

    let asset = starknet_types::Asset::Strk;

    let on_chain_amount = U256::from(128).checked_mul(asset.scale_factor()).unwrap() / 1000;

    let melt_quote_response = node_client
        .melt_quote(ClientMeltQuoteRequest {
            method: method.clone(),
            unit: Unit::MilliStrk.to_string(),
            request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                payee,
                asset,
                amount: on_chain_amount.into(),
            })?,
        })
        .await?;

    let melt_quote_id = melt_quote_response.quote;
    let mut melt_requests = Vec::new();

    for proof in proofs {
        let melt_request = nuts::nut05::MeltRequest {
            quote: melt_quote_id.clone(),
            inputs: vec![proof],
        };

        melt_requests.push(make_melt(node_client.clone(), melt_request));
    }

    let res = join_all(melt_requests).await;
    let ok_vec: Vec<&nuts::nut05::MeltResponse> =
        res.iter().filter_map(|res| res.as_ref().ok()).collect();

    println!("success: {}", ok_vec.len());

    if ok_vec.len() != 1 {
        return Err(crate::common::error::ConcurrencyError::Melt)?;
    }

    // Wait for payment to go through
    loop {
        let response = node_client
            .melt_quote_state(method.clone(), melt_quote_id.clone())
            .await?;

        if response.state == nuts::nut05::MeltQuoteState::Paid {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}
