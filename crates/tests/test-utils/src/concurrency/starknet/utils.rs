<<<<<<< HEAD
use std::str::FromStr;

use node_client::{
    AcknowledgeRequest, GetKeysetsRequest, MeltRequest, MeltResponse, MintQuoteRequest,
    MintQuoteResponse, MintQuoteState, MintRequest, MintResponse, NodeClient, QuoteStateRequest,
    SwapRequest, SwapResponse,
};
use nuts::{
    Amount,
    nut00::{BlindedMessage, secret::Secret},
    nut01::PublicKey,
    nut02::KeysetId,
    nut19::{hash_melt_request, hash_mint_request, hash_swap_request},
};
use starknet_types::{DepositPayload, Unit, constants::ON_CHAIN_CONSTANTS};
use tonic::transport::Channel;
use uuid::Uuid;
=======
use cashu_client::{CashuClient, ClientKeyset, ClientMintQuoteRequest};
use nuts::{
    Amount,
    nut19::{hash_melt_request, hash_mint_request, hash_swap_request},
};
use starknet_types::{DepositPayload, Unit, constants::ON_CHAIN_CONSTANTS};
>>>>>>> origin/HEAD

use anyhow::Result;

use crate::common::utils::{EnvVariables, starknet::pay_invoices};

pub async fn make_mint(
<<<<<<< HEAD
    req: MintRequest,
    mut node_client: NodeClient<Channel>,
) -> Result<MintResponse> {
    let mint_response = node_client.mint(req.clone()).await?.into_inner();

    let outputs: Vec<nuts::nut00::BlindedMessage> = req
        .outputs
        .into_iter()
        .map(|bm| -> Result<nuts::nut00::BlindedMessage> {
            Ok(nuts::nut00::BlindedMessage {
                amount: bm.amount.into(),
                keyset_id: KeysetId::from_bytes(&bm.keyset_id)
                    .map_err(|e| Error::Other(e.into()))?,
                blinded_secret: PublicKey::from_slice(&bm.blinded_secret)
                    .map_err(|e| Error::Other(e.into()))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let nut_mint_request = nuts::nut04::MintRequest {
        quote: Uuid::from_str(&req.quote).map_err(|e| Error::Other(e.into()))?,
        outputs,
    };

    let request_hash = hash_mint_request(&nut_mint_request);
=======
    req: nuts::nut04::MintRequest<String>,
    mut node_client: impl CashuClient,
) -> Result<nuts::nut04::MintResponse> {
    let mint_response = node_client
        .mint(req.clone(), "starknet".to_string())
        .await?;
    let request_hash = hash_mint_request(&req);
>>>>>>> origin/HEAD
    node_client
        .acknowledge("mint".to_string(), request_hash)
        .await?;
    Ok(mint_response)
}

pub async fn make_swap(
<<<<<<< HEAD
    mut node_client: NodeClient<Channel>,
    swap_request: SwapRequest,
) -> Result<SwapResponse> {
    let original_swap_response = node_client.swap(swap_request.clone()).await?.into_inner();

    let inputs: Vec<nuts::nut00::Proof> = swap_request
        .inputs
        .into_iter()
        .map(|p| -> Result<nuts::nut00::Proof> {
            Ok(nuts::nut00::Proof {
                amount: p.amount.into(),
                keyset_id: KeysetId::from_bytes(&p.keyset_id)
                    .map_err(|e| Error::Other(e.into()))?,
                secret: Secret::new(p.secret).map_err(|e| Error::Other(e.into()))?,
                c: PublicKey::from_slice(&p.unblind_signature)
                    .map_err(|e| Error::Other(e.into()))?,
            })
        })
        .collect::<Result<Vec<nuts::nut00::Proof>>>()?;
    let outputs: Vec<nuts::nut00::BlindedMessage> = swap_request
        .outputs
        .into_iter()
        .map(|bm| -> Result<nuts::nut00::BlindedMessage> {
            Ok(nuts::nut00::BlindedMessage {
                amount: bm.amount.into(),
                keyset_id: KeysetId::from_bytes(&bm.keyset_id)
                    .map_err(|e| Error::Other(e.into()))?,
                blinded_secret: PublicKey::from_slice(&bm.blinded_secret)
                    .map_err(|e| Error::Other(e.into()))?,
            })
        })
        .collect::<Result<Vec<nuts::nut00::BlindedMessage>>>()?;
    let nut_swap_request = nuts::nut03::SwapRequest {
        inputs: inputs.clone(),
        outputs: outputs.clone(),
    };

    let request_hash = hash_swap_request(&nut_swap_request);
=======
    mut node_client: impl CashuClient,
    swap_request: nuts::nut03::SwapRequest,
) -> Result<nuts::nut03::SwapResponse> {
    let original_swap_response = node_client.swap(swap_request.clone()).await?;
    let request_hash = hash_swap_request(&swap_request);
>>>>>>> origin/HEAD
    node_client
        .acknowledge("swap".to_string(), request_hash)
        .await?;
    Ok(original_swap_response)
}

pub async fn make_melt(
<<<<<<< HEAD
    mut node_client: NodeClient<Channel>,
    melt_request: MeltRequest,
) -> Result<MeltResponse> {
    let original_melt_response = node_client.melt(melt_request.clone()).await?.into_inner();

    let inputs = melt_request
        .clone()
        .inputs
        .into_iter()
        .map(|p| -> Result<nuts::nut00::Proof> {
            Ok(nuts::nut00::Proof {
                amount: p.amount.into(),
                keyset_id: KeysetId::from_bytes(&p.keyset_id)
                    .map_err(|e| Error::Other(e.into()))?,
                secret: Secret::new(p.secret).map_err(|e| Error::Other(e.into()))?,
                c: PublicKey::from_slice(&p.unblind_signature)
                    .map_err(|e| Error::Other(e.into()))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let nut_melt_request = nuts::nut05::MeltRequest {
        quote: Uuid::from_str(&melt_request.quote).map_err(|e| Error::Other(e.into()))?,
        inputs,
    };

    let request_hash = hash_melt_request(&nut_melt_request);
=======
    mut node_client: impl CashuClient,
    melt_request: nuts::nut05::MeltRequest<String>,
) -> Result<nuts::nut05::MeltResponse> {
    let original_melt_response = node_client
        .melt("starknet".to_string(), melt_request.clone())
        .await?;
    let request_hash = hash_melt_request(&melt_request);
>>>>>>> origin/HEAD
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
