use cashu_client::{CashuClient, ClientKeyset, ClientMintQuoteRequest};
use nuts::{
    Amount,
    nut19::{hash_melt_request, hash_mint_request, hash_swap_request},
};
use starknet_types::{DepositPayload, Unit, constants::ON_CHAIN_CONSTANTS};

use anyhow::Result;

use crate::common::utils::{EnvVariables, starknet::pay_invoices};

pub async fn make_mint(
    req: nuts::nut04::MintRequest<String>,
    mut node_client: impl CashuClient,
) -> Result<nuts::nut04::MintResponse> {
    let mint_response = node_client
        .mint(req.clone(), "starknet".to_string())
        .await?;
    let request_hash = hash_mint_request(&req);
    node_client
        .acknowledge("mint".to_string(), request_hash)
        .await?;
    Ok(mint_response)
}

pub async fn make_swap(
    mut node_client: impl CashuClient,
    swap_request: nuts::nut03::SwapRequest,
) -> Result<nuts::nut03::SwapResponse> {
    let original_swap_response = node_client.swap(swap_request.clone()).await?;
    let request_hash = hash_swap_request(&swap_request);
    node_client
        .acknowledge("swap".to_string(), request_hash)
        .await?;
    Ok(original_swap_response)
}

pub async fn make_melt(
    mut node_client: impl CashuClient,
    melt_request: nuts::nut05::MeltRequest<String>,
) -> Result<nuts::nut05::MeltResponse> {
    let original_melt_response = node_client
        .melt("starknet".to_string(), melt_request.clone())
        .await?;
    let request_hash = hash_melt_request(&melt_request);
    node_client
        .acknowledge("melt".to_string(), request_hash)
        .await?;

    Ok(original_melt_response)
}

pub async fn wait_transac(
    mut node_client: impl CashuClient,
    quote: &nuts::nut04::MintQuoteResponse<String>,
) -> Result<()> {
    loop {
        let response = node_client
            .mint_quote_state("starknet".to_string(), quote.quote.clone())
            .await;

        match response {
            Ok(response) => {
                if response.state == nuts::nut04::MintQuoteState::Paid {
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
    node_client: &mut impl CashuClient,
    unit: &str,
) -> Result<ClientKeyset> {
    let keysets = node_client.keysets().await?.keysets;
    keysets
        .into_iter()
        .find(|ks| ks.active && ks.unit == unit)
        .ok_or_else(|| anyhow::Error::msg("No active keyset found"))
}

pub async fn mint_quote_and_deposit_and_wait(
    mut node_client: impl CashuClient,
    env: EnvVariables,
    amount: Amount,
) -> Result<nuts::nut04::MintQuoteResponse<String>> {
    let mint_quote_request = ClientMintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };

    let quote = node_client.mint_quote(mint_quote_request).await?;

    let on_chain_constants = ON_CHAIN_CONSTANTS.get(env.chain_id.as_str()).unwrap();
    let deposit_payload: DepositPayload = serde_json::from_str(&quote.request)?;
    pay_invoices(
        deposit_payload
            .call_data
            .to_starknet_calls(on_chain_constants.invoice_payment_contract_address)
            .to_vec(),
        env.clone(),
    )
    .await?;

    wait_transac(node_client.clone(), &quote).await?;
    Ok(quote)
}
