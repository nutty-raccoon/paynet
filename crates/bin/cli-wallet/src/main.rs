use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueHint};
use node::{MintQuoteState, NodeClient};
use nuts::nut00;
use rusqlite::Connection;
use starknet_types_core::felt::Felt;
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, value_hint(ValueHint::FilePath))]
    db_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    AddNode {
        #[arg(long, short)]
        node_url: String,
    },
    /// Mint new tokens
    Mint {
        #[arg(long, short)]
        amount: u64,
        #[arg(long, short)]
        unit: String,
        #[arg(long, short)]
        node_id: u32,
    },
    /// Melt (burn) existing tokens
    Melt {
        #[arg(long, short)]
        amount: u64,
        #[arg(long, short)]
        unit: String,
        #[arg(long, short)]
        node_id: u32,
    },
    /// Send tokens
    Send {
        #[arg(long, short)]
        amount: u64,
        #[arg(long, short)]
        unit: String,
        #[arg(long, short)]
        node_id: u32,
    },
    /// Receive tokens
    Receive {
        #[arg(long, short)]
        tokens: String,
        #[arg(long, short)]
        node_id: u32,
    },
}
const STARKNET_METHOD: &str = "starknet";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let db_path = cli
        .db_path
        .or(dirs::data_dir().map(|mut dp| {
            dp.push("cli-wallet.sqlite3");
            dp
        }))
        .ok_or(anyhow!("couldn't find `data_dir` on this computer"))?;
    println!("using database at `{:?}`", db_path);

    let mut db_conn = rusqlite::Connection::open(db_path)?;

    wallet::db::create_tables(&mut db_conn)?;

    match cli.command {
        Commands::AddNode { node_url } => {
            let _node_client = NodeClient::connect(node_url.clone()).await?;
            let node_id = wallet::db::insert_node(&db_conn, &node_url)?;
            println!(
                "Successfully registered {} as node with id `{}`",
                &node_url, node_id
            );
        }
        Commands::Mint {
            amount,
            unit,
            node_id,
        } => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Requesting {} to mint {} {}", &node_url, amount, unit);

            wallet::refresh_node_keysets(&mut db_conn, &mut node_client, node_id).await?;
            // Add mint logic here
            let mint_quote_response = wallet::create_mint_quote(
                &mut db_conn,
                &mut node_client,
                STARKNET_METHOD.to_string(),
                amount,
                unit.clone(),
            )
            .await?;

            println!(
                "MintQuote created with id: {}\nProceed to payment:\n{}",
                &mint_quote_response.quote, &mint_quote_response.request
            );

            loop {
                // Wait a bit
                tokio::time::sleep(Duration::from_secs(1)).await;

                let state = wallet::get_mint_quote_state(
                    &mut db_conn,
                    &mut node_client,
                    STARKNET_METHOD.to_string(),
                    mint_quote_response.quote.clone(),
                )
                .await?;

                if state == MintQuoteState::MnqsPaid {
                    println!("On-chain deposit received");
                    break;
                }
            }

            wallet::mint_and_store_new_tokens(
                &mut db_conn,
                &mut node_client,
                STARKNET_METHOD.to_string(),
                mint_quote_response.quote,
                node_id,
                &unit,
                amount,
            )
            .await?;
            // TODO: remove mint_quote
            println!("Token stored. Finished.");
        }
        Commands::Melt {
            amount,
            unit,
            node_id,
        } => {
            let (mut node_client, _node_url) = connect_to_node(&mut db_conn, node_id).await?;

            println!("Melting {} tokens from {}", amount, unit);
            // Add melt logic here

            let tokens = wallet::fetch_send_inputs_from_db(
                &db_conn,
                &mut node_client,
                node_id,
                amount,
                &unit,
            )
            .await?;

            let inputs = match tokens {
                Some(proof_vector) => proof_vector,
                None => Err(anyhow!("not enough funds"))?,
            };

            let resp = node_client
                .melt(node::MeltRequest {
                    method: "starknet".to_string(),
                    unit,
                    request: serde_json::to_string(&starknet_types::MeltPaymentRequest {
                        recipient: Felt::from_hex_unchecked("0x123"),
                        asset: starknet_types::Asset::Strk,
                        amount: starknet_types::StarknetU256 {
                            high: Felt::ZERO,
                            low: Felt::from_hex_unchecked("0x123"),
                        },
                    })?,
                    inputs: wallet::convert_inputs(&inputs),
                })
                .await?
                .into_inner();

            const INSERT_MELT_RESPONSE: &str = r#"
            INSERT INTO melt_response (
                id, amount, fee, state, expiry
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            "#;

            db_conn.execute(
                INSERT_MELT_RESPONSE,
                [
                    &resp.quote,
                    &resp.amount.to_string(),
                    &resp.fee.to_string(),
                    &resp.state.to_string(),
                    &resp.expiry.to_string(),
                ],
            )?;
        }
        Commands::Send {
            amount,
            unit,
            node_id,
        } => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Sending {} {} using node {}", amount, unit, &node_url);
            let tokens = wallet::fetch_send_inputs_from_db(
                &db_conn,
                &mut node_client,
                node_id,
                amount,
                &unit,
            )
            .await?;

            match tokens {
                Some(tokens) => {
                    let s = serde_json::to_string(&tokens)?;
                    println!("Tokens:\n{}", s);
                }
                None => println!("Not enough funds"),
            }
        }
        Commands::Receive { tokens, node_id } => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            let tokens: Vec<nut00::Proof> = serde_json::from_str(&tokens)?;

            println!("Receiving tokens on `{}`", node_url);
            wallet::receive_tokens(&db_conn, &mut node_client, node_id, tokens).await?;
            println!("Finished");
        }
    }

    Ok(())
}

pub async fn connect_to_node(
    conn: &mut Connection,
    node_id: u32,
) -> Result<(NodeClient<tonic::transport::Channel>, String)> {
    let node_url = wallet::db::get_node_url(conn, node_id)?
        .ok_or_else(|| anyhow!("no node with id {node_id}"))?;
    let node_client = NodeClient::connect(node_url.clone()).await?;
    Ok((node_client, node_url))
}
