use anyhow::Result;
use node_client::{
    AcknowledgeRequest, BlindedMessage, GetKeysRequest, GetKeysetsRequest, MeltQuoteRequest,
    MeltRequest, MintQuoteRequest, MintRequest, Proof, SwapRequest, hash_melt_request,
    hash_mint_request, hash_swap_request,
};
use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
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
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let original_mint_quote_response = client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();
    // Not cached
    let second_mint_quote_response = client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();
    assert_ne!(original_mint_quote_response, second_mint_quote_response);

    // MINT
    let keysets = client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: original_mint_quote_response.quote,
        outputs: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        }],
    };
    let original_mint_response = client.mint(mint_request.clone()).await?.into_inner();
    let cached_mint_response = client.mint(mint_request.clone()).await?.into_inner();
    assert_eq!(original_mint_response, cached_mint_response);
    let request_hash = hash_mint_request(&mint_request);
    client
        .acknowledge(AcknowledgeRequest {
            path: "mint".to_string(),
            request_hash,
        })
        .await?;
    let post_ack_mint_response = client.mint(mint_request).await;
    assert!(post_ack_mint_response.is_err());

    // SWAP
    let node_pubkey_for_amount = PublicKey::from_hex(
        &client
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
    )?;
    let blind_signature = PublicKey::from_slice(
        &original_mint_response
            .signatures
            .first()
            .unwrap()
            .blind_signature,
    )
    .unwrap();
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = Proof {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        secret: secret.to_string(),
        unblind_signature: unblinded_signature.to_bytes().to_vec(),
    };

    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let blind_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let swap_request = SwapRequest {
        inputs: vec![proof],
        outputs: vec![blind_message],
    };
    let original_swap_response = client.swap(swap_request.clone()).await?.into_inner();
    let cached_swap_response = client.swap(swap_request.clone()).await?.into_inner();
    assert_eq!(original_swap_response, cached_swap_response);

    let request_hash = hash_swap_request(&swap_request);
    client
        .acknowledge(AcknowledgeRequest {
            path: "swap".to_string(),
            request_hash,
        })
        .await?;
    let post_ack_swap_response = client.swap(swap_request).await;
    assert!(post_ack_swap_response.is_err());

    // MELT
    let blind_signature = PublicKey::from_slice(
        &original_swap_response
            .signatures
            .first()
            .unwrap()
            .blind_signature,
    )
    .unwrap();
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = Proof {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        secret: secret.to_string(),
        unblind_signature: unblinded_signature.to_bytes().to_vec(),
    };

    let melt_quote_request = MeltQuoteRequest {
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

    let melt_quote_response = client.melt_quote(melt_quote_request).await?.into_inner();

    // Now test melt operation with the quote (this should be cached)
    let melt_request = MeltRequest {
        quote: melt_quote_response.quote,
        method: "starknet".to_string(),
        inputs: vec![proof],
    };
    let original_melt_response = client.melt(melt_request.clone()).await?.into_inner();
    let cached_melt_response = client.melt(melt_request.clone()).await?.into_inner();
    assert_eq!(original_melt_response, cached_melt_response);
    let request_hash = hash_melt_request(&melt_request);
    client
        .acknowledge(AcknowledgeRequest {
            path: "melt".to_string(),
            request_hash,
        })
        .await?;
    let post_ack_melt_response = client.melt(melt_request).await;
    assert!(post_ack_melt_response.is_err());

    Ok(())
}
