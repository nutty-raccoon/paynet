use crate::env_variables::EnvVariables;
use anyhow::{Error, Result, anyhow};
use itertools::Itertools;
use log::error;
use node::{MeltRequest, MintQuoteState, NodeClient, QuoteStateRequest, hash_melt_request};
use primitive_types::U256;
use rusqlite::Connection;
use serde_json;
use starknet::accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount};
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::signers::{LocalWallet, SigningKey};
use starknet_types::{Asset, STARKNET_STR, Unit};
use starknet_types_core::felt::Felt;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::info;
use url::Url;
use wallet::types::NodeUrl;
use wallet::{
    self,
    types::compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
};

fn init_account(
    env: EnvVariables,
) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>, Error> {
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(
        &env.private_key,
    )?));
    let address = Felt::from_hex(&env.account_address)?;

    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&env.rpc_url)?));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        Felt::from_bytes_be_slice("SN_DEVNET".as_bytes()),
        ExecutionEncoding::New,
    );

    Ok(account)
}

pub async fn mint(
    db_conn: &mut Connection,
    node_id: u32,
    mut node_client: NodeClient<Channel>,
    amount: U256,
    asset: Asset,
    env: EnvVariables,
) -> Result<()> {
    let amount = amount
        .checked_mul(asset.precision())
        .ok_or(anyhow!("amount too big"))?;
    let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

    let tx = db_conn.transaction()?;

    let quote = wallet::create_mint_quote(
        &tx,
        &mut node_client,
        node_id,
        STARKNET_STR.to_string(),
        amount,
        unit.as_str(),
    )
    .await?;

    pay_invoice(quote.request, env).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let state = match wallet::get_mint_quote_state(
            &tx,
            &mut node_client,
            STARKNET_STR.to_string(),
            quote.quote.clone(),
        )
        .await?
        {
            Some(s) => s,
            None => {
                info!("quote expired");
                return Ok(());
            }
        };
        if state == MintQuoteState::MnqsPaid {
            break;
        }
    }

    wallet::mint_and_store_new_tokens(
        &tx,
        &mut node_client,
        STARKNET_STR.to_string(),
        quote.quote,
        node_id,
        unit.as_str(),
        amount,
    )
    .await?;
    Ok(())
}

pub async fn pay_invoice(invoice_json: String, env: EnvVariables) -> Result<()> {
    let account = init_account(env)?;

    let calls: [starknet_types::Call; 2] = serde_json::from_str(&invoice_json)?;
    let tx_hash = account
        .execute_v3(calls.into_iter().map(Into::into).collect())
        .send()
        .await
        .inspect_err(|e| error!("send payment tx failed: {:?}", e))?
        .transaction_hash;

    watch_tx(account.provider(), tx_hash).await?;

    Ok(())
}

pub async fn watch_tx<P>(provider: P, transaction_hash: Felt) -> Result<()>
where
    P: starknet::providers::Provider,
{
    loop {
        use starknet::core::types::{StarknetError, TransactionExecutionStatus, TransactionStatus};
        use starknet::providers::ProviderError;
        match provider.get_transaction_status(transaction_hash).await {
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Succeeded)) => {
                return Ok(());
            }
            Ok(TransactionStatus::AcceptedOnL2(TransactionExecutionStatus::Reverted)) => {
                return Err(anyhow!("tx reverted"));
            }
            Ok(TransactionStatus::Received) => {}
            Ok(TransactionStatus::Rejected) => return Err(anyhow!("tx rejected")),
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {}
            Err(err) => return Err(err.into()),
            Ok(TransactionStatus::AcceptedOnL1(_)) => unreachable!(),
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

pub async fn send(
    db_conn: &mut Connection,
    node_id: u32,
    mut node_client: NodeClient<Channel>,
    node_url: NodeUrl,
    amount: U256,
    asset: Asset,
    memo: Option<String>,
) -> Result<CompactWad<Unit>> {
    let amount = amount
        .checked_mul(asset.precision())
        .ok_or(anyhow!("amount too big"))?;
    let (amount, unit, _) = asset.convert_to_amount_and_unit(amount)?;
    let tx = db_conn.transaction()?;
    let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
        &tx,
        &mut node_client,
        node_id,
        amount,
        unit.as_str(),
    )
    .await?
    .ok_or(anyhow!("not enough funds"))?;
    tx.commit()?;

    let tx = db_conn.transaction()?;
    let proofs = wallet::load_tokens_from_db(&tx, proofs_ids).await?;
    let compact_proofs = proofs
        .into_iter()
        .chunk_by(|p| p.keyset_id)
        .into_iter()
        .map(|(keyset_id, proofs)| CompactKeysetProofs {
            keyset_id,
            proofs: proofs
                .map(|p| CompactProof {
                    amount: p.amount,
                    secret: p.secret,
                    c: p.c,
                })
                .collect(),
        })
        .collect();
    let wad = CompactWad {
        node_url,
        unit,
        memo,
        proofs: compact_proofs,
    };
    tx.commit()?;
    // println!("{}", wad.to_string());
    Ok(wad)
}

pub async fn receive(
    db_conn: &mut Connection,
    node_id: u32,
    mut node_client: NodeClient<Channel>,
    wad: &CompactWad<Unit>,
) -> Result<()> {
    wallet::receive_wad(
        db_conn,
        &mut node_client,
        node_id,
        wad.unit.as_str(),
        &wad.proofs,
    )
    .await?;
    Ok(())
}

const STARKNET_METHOD: &str = "starknet";

pub async fn melt(
    db_conn: &mut Connection,
    node_id: u32,
    mut node_client: NodeClient<Channel>,
    amount: U256,
    asset: Asset,
    to: String,
) -> Result<()> {
    let amount = amount
        .checked_mul(asset.precision())
        .ok_or(anyhow!("amount too big"))?;
    let (amount, unit, _) = asset.convert_to_amount_and_unit(amount)?;

    let tx = db_conn.transaction()?;
    let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
        &tx,
        &mut node_client,
        node_id,
        amount,
        unit.as_str(),
    )
    .await?
    .ok_or(anyhow!("not enough funds"))?;
    tx.commit()?;

    let tx = db_conn.transaction()?;
    let inputs = wallet::load_tokens_from_db(&tx, proofs_ids).await?;
    let payee_address = Felt::from_hex(&to)?;
    if !starknet_types::is_valid_starknet_address(&payee_address) {
        return Err(anyhow!("Invalid starknet address: {}", payee_address));
    }
    let melt_request = MeltRequest {
        method: STARKNET_STR.to_string(),
        unit: unit.to_string(),
        request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
            payee: payee_address,
            asset: starknet_types::Asset::Strk,
        })?,
        inputs: wallet::convert_inputs(&inputs),
    };
    let melt_request_hash = hash_melt_request(&melt_request);
    let resp = node_client.melt(melt_request).await?.into_inner();
    wallet::db::register_melt_quote(&tx, node_id, &resp)?;
    tx.commit()?;
    wallet::acknowledge(
        &mut node_client,
        nuts::nut19::Route::Melt,
        melt_request_hash,
    )
    .await?;

    loop {
        let melt_quote_state_response = node_client
            .melt_quote_state(QuoteStateRequest {
                method: STARKNET_METHOD.to_string(),
                quote: resp.quote.clone(),
            })
            .await?
            .into_inner();

        if !melt_quote_state_response.transfer_ids.is_empty() {
            println!(
                "{}",
                format_melt_transfers_id_into_term_message(melt_quote_state_response.transfer_ids)
            );
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(())
}

fn format_melt_transfers_id_into_term_message(transfer_ids: Vec<String>) -> String {
    let mut string_to_print = "Melt done. Withdrawal settled with tx".to_string();
    if transfer_ids.len() != 1 {
        string_to_print.push('s');
    }
    string_to_print.push_str(": ");
    let mut iterator = transfer_ids.into_iter();
    string_to_print.push_str(&iterator.next().unwrap());
    for tx_hash in iterator {
        string_to_print.push_str(", ");
        string_to_print.push_str(&tx_hash);
    }

    string_to_print
}
