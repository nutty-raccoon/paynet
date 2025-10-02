use anyhow::Result;
use cashu_client::{CashuClient, ClientMeltQuoteRequest, ClientMintQuoteRequest};
use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut02::KeysetId;
use nuts::nut19::{hash_melt_request, hash_mint_request, hash_swap_request};
use starknet_liquidity_source::MeltPaymentRequest;
use starknet_types::{StarknetU256, Unit};
use starknet_types_core::felt::Felt;

// This tests check that the route that we want to cache are indeed cached.
//
// Mint Quote (no cache):
// - call mint_quote with a request
// - call it again with same request and check that it gets a different quote
//
// Mint (cache):
// - call mint with a request
// - call it again and check the response is the same
// - call acknowledge on the response
// - call it again and check the response is an error
//
// Swap (cache):
// - call swap with a request
// - call it again and check the response is the same
// - call acknowledge on the response
// - call it again and check the response is an error
//
// Melt (cache):
// - call melt_quote to get a quote (not cached)
// - call melt with a request
// - call it again and check the response is the same
// - call acknowledge on the response
// - call it again and check the response is an error
#[tokio::test]
async fn works() -> Result<()> {
    let mut client = init_node_client().await?;
    let amount = Amount::from_i64_repr(32);

    // MINT QUOTE
    let mint_quote_request = ClientMintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let original_mint_quote_response = client.mint_quote(mint_quote_request.clone()).await?;
    // Not cached
    let second_mint_quote_response = client.mint_quote(mint_quote_request).await?;
    assert_ne!(original_mint_quote_response, second_mint_quote_response);

    // MINT
    let keysets = client.keysets().await?.keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let mint_request = nuts::nut04::MintRequest {
        quote: original_mint_quote_response.quote,
        outputs: vec![nuts::nut00::BlindedMessage {
            amount: amount.into(),
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret: blinded_secret,
        }],
    };
    let original_mint_response = client
        .mint(mint_request.clone(), "starknet".to_string())
        .await?;
    let cached_mint_response = client
        .mint(mint_request.clone(), "starknet".to_string())
        .await?;
    assert_eq!(original_mint_response, cached_mint_response);
    let request_hash = hash_mint_request(&mint_request);
    client.acknowledge("mint".to_string(), request_hash).await?;
    let post_ack_mint_response = client.mint(mint_request, "starknet".to_string()).await;
    assert!(post_ack_mint_response.is_err());

    // SWAP
    let node_pubkey_for_amount = client
        .keys(Some(active_keyset.id.clone()))
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
        amount: amount.into(),
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        secret: secret,
        c: unblinded_signature,
    };

    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let blind_message = nuts::nut00::BlindedMessage {
        amount: amount.into(),
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        blinded_secret: blinded_secret,
    };

    let swap_request = nuts::nut03::SwapRequest {
        inputs: vec![proof],
        outputs: vec![blind_message],
    };
    let original_swap_response = client.swap(swap_request.clone()).await?;
    let cached_swap_response = client.swap(swap_request.clone()).await?;
    assert_eq!(original_swap_response, cached_swap_response);

    let request_hash = hash_swap_request(&swap_request);
    client.acknowledge("swap".to_string(), request_hash).await?;
    let post_ack_swap_response = client.swap(swap_request).await;
    assert!(post_ack_swap_response.is_err());

    // MELT
    let blind_signature = original_swap_response.signatures.first().unwrap().c;
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = nuts::nut00::Proof {
        amount: amount.into(),
        keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
        secret: secret,
        c: unblinded_signature,
    };

    let melt_quote_request = ClientMeltQuoteRequest {
        method: "starknet".to_string(),
        unit: Unit::MilliStrk.to_string(),
        request: serde_json::to_string(&MeltPaymentRequest {
            payee: Felt::from_hex_unchecked(
                "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691",
            ),
            asset: starknet_types::Asset::Strk,
            amount: StarknetU256 {
                low: Felt::from_dec_str("32000000000000000").unwrap(),
                high: Felt::from(0),
            },
        })
        .unwrap(),
    };

    let melt_quote_response = client.melt_quote(melt_quote_request).await?;

    // Now test melt operation with the quote (this should be cached)
    let melt_request = nuts::nut05::MeltRequest {
        quote: melt_quote_response.quote,
        inputs: vec![proof],
    };
    let original_melt_response = client
        .melt("starknet".to_string(), melt_request.clone())
        .await?;
    let cached_melt_response = client
        .melt("starknet".to_string(), melt_request.clone())
        .await?;
    assert_eq!(original_melt_response, cached_melt_response);
    let request_hash = hash_melt_request(&melt_request);
    client.acknowledge("melt".to_string(), request_hash).await?;
    let post_ack_melt_response = client.melt("starknet".to_string(), melt_request).await;
    assert!(post_ack_melt_response.is_err());

    Ok(())
}
