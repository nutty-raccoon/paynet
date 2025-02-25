use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueHint};
use node::{MintQuoteState, NodeClient};
use rusqlite::Connection;
use starknet_types_core::felt::Felt;
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::EnvFilter;
use wallet::types::Wad;

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
    /// Register a new node
    AddNode {
        /// Url of the node
        #[arg(long, short)]
        node_url: String,
    },
    /// Mint new tokens
    Mint {
        /// Amount requested
        #[arg(long, short)]
        amount: u64,
        /// Unit requested
        #[arg(long, short)]
        unit: String,
        /// Id of the node to use
        #[arg(long, short)]
        node_id: u32,
    },
    /// Melt existing tokens
    Melt {
        /// Amount to melt
        #[arg(long, short)]
        amount: u64,
        /// Unit to melt
        #[arg(long, short)]
        unit: String,
        /// Id of the node to use
        #[arg(long, short)]
        node_id: u32,
    },
    /// Send tokens
    Send {
        /// Amount to send
        #[arg(long, short)]
        amount: u64,
        /// Unit to send
        #[arg(long, short)]
        unit: String,
        /// Id of the node to use
        #[arg(long, short)]
        node_id: u32,
    },
    /// Receive a wad of proofs
    Receive {
        /// Json string of the wad
        #[arg(long, short)]
        wad_as_json: String,
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
            let tx = db_conn.transaction()?;
            let (mut _node_client, node_id) = wallet::register_node(&tx, node_url.clone()).await?;
            tx.commit()?;
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

            let tx = db_conn.transaction()?;
            // Add mint logic here
            let mint_quote_response = wallet::create_mint_quote(
                &tx,
                &mut node_client,
                STARKNET_METHOD.to_string(),
                amount,
                unit.clone(),
            )
            .await?;
            tx.commit()?;

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

            let tx = db_conn.transaction()?;
            wallet::mint_and_store_new_tokens(
                &tx,
                &mut node_client,
                STARKNET_METHOD.to_string(),
                mint_quote_response.quote,
                node_id,
                &unit,
                amount,
            )
            .await?;
            tx.commit()?;

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

            let tx = db_conn.transaction()?;
            let tokens =
                wallet::fetch_inputs_from_db_or_node(&tx, &mut node_client, node_id, amount, &unit)
                    .await?;
            tx.commit()?;

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

            wallet::db::register_melt_quote(&db_conn, &resp)?;
        }
        Commands::Send {
            amount,
            unit,
            node_id,
        } => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Sending {} {} using node {}", amount, unit, &node_url);

            let tx = db_conn.transaction()?;
            let opt_proofs =
                wallet::fetch_inputs_from_db_or_node(&tx, &mut node_client, node_id, amount, &unit)
                    .await?;
            tx.commit()?;

            let wad = match opt_proofs {
                Some(proofs) => Wad { node_url, proofs },
                None => {
                    println!("Not enough funds");
                    return Ok(());
                }
            };
            println!("Wad:\n{}", serde_json::to_string(&wad)?);
        }
        Commands::Receive { wad_as_json } => {
            let wad: Wad = serde_json::from_str(&wad_as_json)?;
            let (mut node_client, node_id) = wallet::register_node(&db_conn, wad.node_url).await?;

            println!("Receiving tokens on node `{}`", node_id);
            let tx = db_conn.transaction()?;
            wallet::receive_wad(&tx, &mut node_client, node_id, wad.proofs).await?;
            tx.commit()?;
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
