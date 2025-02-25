use anyhow::{Result, anyhow};
use node::{MintQuoteState, NodeClient};
use rusqlite::Connection;
use std::{path::PathBuf, time::Duration};
use tracing::info;
use tracing_subscriber::EnvFilter;

use clap::{Parser, Subcommand, ValueHint};

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
    info!("database located at `{:?}`", db_path);

    let mut db_conn = rusqlite::Connection::open(db_path)?;

    wallet::db::create_tables(&mut db_conn)?;

    match cli.command {
        Commands::AddNode { node_url } => {
            let _node_client = NodeClient::connect(node_url.clone()).await?;
            let _ = wallet::db::insert_node(&db_conn, &node_url)?;
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
            println!("Token stored. Finised.");
        }
        Commands::Melt { amount, unit } => {
            println!("Melting {} tokens from {}", amount, unit);
            // Add melt logic here
        }
        Commands::Send {
            amount,
            unit,
            node_id,
        } => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Sending {} {} using mint {}", amount, unit, &node_url);
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
                    println!("{}", s);
                }
                None => println!("Not enough funds"),
            }
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
