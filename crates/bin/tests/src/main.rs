use std::{path::PathBuf, str::FromStr};

use anyhow::{Error, anyhow};
use clap::{Parser, arg};
use futures::future::join_all;
use node::{
    AcknowledgeRequest, BlindedMessage, GetKeysRequest, GetKeysetsRequest, MintQuoteRequest,
    MintQuoteState, MintRequest, NodeClient, Proof, QuoteStateRequest, SwapRequest, SwapResponse,
    hash_mint_request, hash_swap_request,
};
use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use rusqlite::Connection;
use starknet_types::Unit;
use tonic::transport::Channel;
use tracing_subscriber::EnvFilter;
use wallet::connect_to_node;
use wallet::types::NodeUrl;

use crate::env_variables::EnvVariables;
use crate::wallet_ops::{melt, mint, pay_invoice, receive, send};

mod env_variables;
mod wallet_ops;

#[derive(clap::Parser)]
#[command(version, about = "Test runner")]
pub struct Cli {
    #[command(subcommand)]
    pub test_type: TestType,
}

#[derive(clap::Subcommand)]
pub enum TestType {
    /// End-to-end tests
    E2e,
    /// Concurrency tests
    Concurrency {
        #[arg(long)]
        operation: Option<Operation>,
        #[arg(long, value_name = "N")]
        count: Option<u32>,
    },
    /// Stress tests
    Stress {
        #[arg(long)]
        operation: Option<Operation>,
        #[arg(long, value_name = "N")]
        count: Option<u32>,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Operation {
    Melt,
    Mint,
    Send,
    Receive,
}

fn db_connection() -> Result<(Connection, PathBuf), Error> {
    let db_path = dirs::data_dir()
        .map(|mut dp| {
            dp.push("cli-wallet.sqlite3");
            dp
        })
        .ok_or(anyhow!("couldn't find `data_dir` on this computer"))?;
    println!(
        "Using database at {:?}\n",
        db_path
            .as_path()
            .to_str()
            .ok_or(anyhow!("invalid db path"))?
    );

    let mut db_conn = rusqlite::Connection::open(db_path.clone())?;

    wallet::db::create_tables(&mut db_conn)?;
    Ok((db_conn, db_path))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let env = env_variables::read_env_variables()?;

    match cli.test_type {
        TestType::E2e => run_e2e(env).await?,
        TestType::Concurrency { operation, count } => {
            run_concurrency(operation, count, env).await?
        }
        TestType::Stress { operation, count } => run_stress(operation, count).await?,
    }

    Ok(())
}

async fn run_e2e(env: EnvVariables) -> anyhow::Result<(), Error> {
    let (mut db_conn, _db_path) = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;

    let tx = db_conn.transaction()?;

    let (node_client, node_id) = wallet::register_node(&tx, node_url.clone()).await?;
    tx.commit()?;

    mint(
        &mut db_conn,
        node_id,
        node_client.clone(),
        10.into(),
        starknet_types::Asset::Strk,
        env,
    )
    .await?;
    let wad = send(
        &mut db_conn,
        node_id,
        node_client.clone(),
        node_url,
        10.into(),
        starknet_types::Asset::Strk,
        Some("./test.wad".to_string()),
    )
    .await?;
    receive(&mut db_conn, node_id, node_client.clone(), &wad).await?;
    melt(
        &mut db_conn,
        node_id,
        node_client.clone(),
        10.into(),
        starknet_types::Asset::Strk,
        "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
    )
    .await?;

    println!("✅ [E2E] Test passed: funds successfully mint, sent, received and melt");
    Ok(())
}

async fn run_concurrency(
    _op: Option<Operation>,
    count: Option<u32>,
    env: EnvVariables,
) -> anyhow::Result<()> {
    println!("[CONCURRENCY] Launching concurrency tests for with {count:?} cases");
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let mut node_client = connect_to_node(&node_url).await?;

    let amount = Amount::from_i64_repr(32);

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let original_mint_quote_response = node_client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

    pay_invoice(original_mint_quote_response.request, env).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let response = node_client
            .mint_quote_state(QuoteStateRequest {
                method: "starknet".to_string(),
                quote: original_mint_quote_response.quote.clone(),
            })
            .await;

        match response {
            Ok(response) => {
                let response = response.into_inner();
                let state = MintQuoteState::try_from(response.state)?;
                if state == MintQuoteState::MnqsPaid {
                    break;
                }
            }
            Err(e) => {
                println!("{e}")
            }
        }
    }

    let keysets = node_client
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

    let original_mint_response = node_client.mint(mint_request.clone()).await?.into_inner();
    let request_hash = hash_mint_request(&mint_request);
    node_client
        .acknowledge(AcknowledgeRequest {
            path: "mint".to_string(),
            request_hash,
        })
        .await?;

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

    let secret = Secret::generate();
    let (blinded_secret, _r) = blind_message(secret.as_bytes(), None)?;
    let blind_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let swap_request = SwapRequest {
        inputs: vec![proof],
        outputs: vec![blind_message],
    };
    let mut multi_swap = Vec::new();
    for _ in 0..100 {
        multi_swap.push(make_swap(node_client.clone(), swap_request.clone()))
    }
    let res = join_all(multi_swap).await;
    let ok_vec: Vec<&SwapResponse> = res.iter().filter_map(|res| res.as_ref().ok()).collect();
    assert_eq!(ok_vec.len(), 1);
    println!("✅ [CONCURRENCY] All tasks completed successfully — concurrency test passed.\n");
    Ok(())
}

async fn make_swap(
    mut node_client: NodeClient<Channel>,
    swap_request: SwapRequest,
) -> Result<SwapResponse, Error> {
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

async fn run_stress(_op: Option<Operation>, count: Option<u32>) -> anyhow::Result<()> {
    println!("Launching stress tests for with {count:?} cases");
    // Implémentation…
    Ok(())
}
