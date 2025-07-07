use anyhow::Result;
use node_client::{
    BlindedMessage, GetKeysRequest, GetKeysetsRequest, MeltRequest, MintQuoteRequest, MintRequest,
    Proof,
};
use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use starknet_liquidity_source::MeltPaymentRequest;
use starknet_types::{Asset, Unit};
use starknet_types_core::felt::Felt;

#[tokio::test]
#[cfg(feature = "starknet")]
async fn test_melt_with_valid_address() -> Result<()> {
    let mut node_client = init_node_client().await?;

    let valid_address = Felt::from_hex_unchecked(
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    );

    let amount = Amount::from_i64_repr(32);

    // MINT
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    // MINT QUOTE
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.as_str().to_string(),
        description: None,
    };
    let original_mint_quote_response = node_client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

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
    let original_mint_response = node_client.mint(mint_request.clone()).await?.into_inner();

    // SWAP
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
    )?;

    // MELT
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

    let payment_request = MeltPaymentRequest {
        payee: valid_address,
        asset: Asset::Strk,
    };

    let serialized_request = serde_json::to_string(&payment_request)?;

    let melt_request = MeltRequest {
        method: "starknet".to_string(),
        unit: Unit::MilliStrk.as_str().to_string(),
        request: serialized_request,
        inputs: vec![proof],
    };

    let result = node_client.melt(melt_request).await;

    match result {
        Err(status) => {
            assert!(
                !status.message().contains("Invalid starknet address"),
                "Address validation should pass, but failed with: {}",
                status.message()
            );
            println!(
                "Test passed: Request failed as expected with error: {}",
                status.message()
            );
        }
        Ok(_) => {
            println!("Test passed but unexpectedly got successful response");
        }
    }

    Ok(())
}

#[tokio::test]
#[cfg(feature = "starknet")]
async fn test_melt_with_invalid_addresses() -> Result<()> {
    let mut node_client = init_node_client().await?;

    // Create test cases for invalid addresses
    let invalid_addresses = [
        // Address 0 (reserved)
        Felt::from(0),
        // Address 1 (reserved)
        Felt::from(1),
        // Address at 2^251 + 17 * 2^192
        Felt::from_hex_unchecked(
            "0x800000000000000000000000000000000000000000000000000000000000000",
        ), // Address above 2^251 + 17 * 2^192
        Felt::from_hex_unchecked(
            "0x800000000000000000000000000000000000000000000000000000000000001",
        ),
    ];

    let amount = Amount::from_i64_repr(32);

    // MINT
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    // MINT QUOTE
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.as_str().to_string(),
        description: None,
    };
    let original_mint_quote_response = node_client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

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
    let original_mint_response = node_client.mint(mint_request.clone()).await?.into_inner();

    // SWAP
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

    for invalid_address in invalid_addresses {
        // MELT
        let payment_request = MeltPaymentRequest {
            payee: invalid_address,
            asset: Asset::Strk,
        };

        let serialized_request = serde_json::to_string(&payment_request)?;

        let melt_request = MeltRequest {
            method: "starknet".to_string(),
            unit: Unit::MilliStrk.to_string(),
            request: serialized_request,
            inputs: vec![proof.clone()],
        };

        let result = node_client.melt(melt_request).await;

        // Validate: It should fail with an "Invalid starknet address" error
        match result {
            Err(status) => {
                assert!(
                    status.message().contains("Invalid starknet address")
                        || status.message().contains("invalid starknet address"),
                    "Expected an invalid address error, but got: {}",
                    status.message()
                );
            }
            Ok(_) => {
                panic!(
                    "Test failed: Request with invalid address {} was accepted",
                    invalid_address
                );
            }
        }
    }

    Ok(())
}
