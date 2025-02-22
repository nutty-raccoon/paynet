use anyhow::Result;
use node::NodeClient;
use node::{MintQuoteRequest, MintQuoteResponse};

pub async fn create_mint_quote(
    node_client: &mut NodeClient<tonic::transport::Channel>,
    method: String,
    amount: u64,
    unit: String,
) -> Result<MintQuoteResponse> {
    let response = node_client
        .mint_quote(MintQuoteRequest {
            method,
            amount,
            unit,
            description: None,
        })
        .await?;

    Ok(response.into_inner())
}
