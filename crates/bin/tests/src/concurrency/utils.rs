use node::{
    AcknowledgeRequest, GetKeysetsRequest, MeltRequest, MeltResponse, MintQuoteRequest,
    MintQuoteResponse, MintQuoteState, MintRequest, MintResponse, NodeClient, QuoteStateRequest,
    SwapRequest, SwapResponse, hash_melt_request, hash_mint_request, hash_swap_request,
};
use nuts::Amount;
use starknet_types::Unit;
use tonic::transport::Channel;

use crate::{
    env_variables::EnvVariables,
    errors::{Error, Result},
    utils::pay_invoices,
};

pub async fn make_mint(
    req: MintRequest,
    mut node_client: NodeClient<Channel>,
) -> Result<MintResponse> {
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

pub async fn make_swap(
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

pub async fn make_melt(
    mut node_client: NodeClient<Channel>,
    melt_request: MeltRequest,
) -> Result<MeltResponse> {
    let original_melt_response = node_client.melt(melt_request.clone()).await?.into_inner();
    let request_hash = hash_melt_request(&melt_request);
    node_client
        .acknowledge(AcknowledgeRequest {
            path: "melt".to_string(),
            request_hash,
        })
        .await?;

    Ok(original_melt_response)
}

pub async fn wait_transac(
    mut node_client: NodeClient<Channel>,
    quote: &MintQuoteResponse,
) -> Result<()> {
    loop {
        let response = node_client
            .mint_quote_state(QuoteStateRequest {
                method: "starknet".to_string(),
                quote: quote.quote.clone(),
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
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

pub async fn get_active_keyset(
    node_client: &mut NodeClient<Channel>,
    unit: &str,
) -> Result<node::Keyset> {
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    keysets
        .into_iter()
        .find(|ks| ks.active && ks.unit == unit)
        .ok_or_else(|| Error::Other(anyhow::Error::msg("No active keyset found")))
}

pub async fn mint_and_wait(
    mut node_client: NodeClient<Channel>,
    env: EnvVariables,
    amount: Amount,
) -> Result<MintQuoteResponse> {
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };

    let quote = node_client
        .mint_quote(mint_quote_request)
        .await?
        .into_inner();

    let calls: [starknet_types::Call; 2] = serde_json::from_str(&quote.request)?;
    pay_invoices(calls.to_vec(), env.clone()).await?;

    wait_transac(node_client.clone(), &quote).await?;
    Ok(quote)
}
