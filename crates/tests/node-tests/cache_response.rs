use anyhow::{Result, anyhow};
use node::{
    AcknowledgeRequest, BlindedMessage, MeltRequest, MintQuoteRequest, MintRequest,
    hash_mint_quote_request,
};
use node_tests::{init_health_client, init_node_client};
use nuts::Amount;
use nuts::nut01::PublicKey;
use nuts::nut02::{KeySetVersion, KeysetId};
use tonic_health::pb::{HealthCheckRequest, health_check_response::ServingStatus};
#[tokio::test]
async fn mint_quote_no_ack() -> Result<()> {
    let mut client = init_node_client().await?;

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: 50,
        unit: "strk".to_string(),
        description: None,
    };

    let res = client.mint_quote(mint_quote_request.clone()).await?;

    let mint_response = res.into_inner();

    let res_2 = client.mint_quote(mint_quote_request).await?;
    let mint_response_2 = res_2.into_inner();
    assert_eq!(mint_response, mint_response_2);

    Ok(())
}

#[tokio::test]
async fn mint_quote_ack() -> Result<()> {
    let mut client = init_node_client().await?;

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: 51,
        unit: "strk".to_string(),
        description: None,
    };
    let request_hash = hash_mint_quote_request(&mint_quote_request);
    let res = client.mint_quote(mint_quote_request.clone()).await?;

    let mint_response = res.into_inner();

    client
        .acknowledge(AcknowledgeRequest {
            path: "/v1/mint/starknet".to_string(),
            request_hash,
        })
        .await?;
    let res_2 = client.mint_quote(mint_quote_request).await?;
    let mint_response_2 = res_2.into_inner();
    assert_ne!(mint_response, mint_response_2);

    Ok(())
}

#[tokio::test]
async fn mint_ack() -> Result<()> {
    let mut client = init_node_client().await?;
    let amount = Amount::from_i64_repr(1000);

    // First, get a valid quote
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into_i64_repr() as u64, // Ensure this matches your Amount
        unit: "strk".to_string(),
        description: None,
    };

    let quote_response = client.mint_quote(mint_quote_request).await?;
    let quote_id = quote_response.into_inner().quote;

    let keyset_id: [u8; 7] = [0, 1, 2, 3, 4, 5, 6];
    let keyset_id = KeysetId::new(KeySetVersion::Version00, keyset_id);
    let blinded_secret =
        PublicKey::from_hex("02194603ffa36356f4a56b7df9371fc3192472351453ec7398b8da8117e7c3e104")
            .unwrap();

    let blinded_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: keyset_id.to_bytes().to_vec(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let outputs = [blinded_message.clone()];

    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: quote_id, // Valid UUID
        outputs: outputs.to_vec(),
    };

    let res = client.mint(mint_request.clone()).await?;

    let mint_response = res.into_inner();

    let res_2 = client.mint(mint_request).await?;
    let mint_response_2 = res_2.into_inner();
    assert_eq!(mint_response, mint_response_2);

    Ok(())
}
