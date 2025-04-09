use anyhow::{Result, anyhow};
use node::{
    AcknowledgeRequest, BlindedMessage, MeltRequest, MintQuoteRequest, MintRequest,
    hash_mint_quote_request,
};
use node_tests::{init_health_client, init_node_client};
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
