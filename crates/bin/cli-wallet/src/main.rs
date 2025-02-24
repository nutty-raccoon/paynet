use anyhow::{Result, anyhow};
use node::{MintQuoteState, NodeClient};
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
    #[arg(long, short, value_hint(ValueHint::Url))]
    node_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Mint new tokens
    Mint {
        #[arg(long, short)]
        amount: u64,
        #[arg(long, short)]
        unit: String,
    },

    /// Melt (burn) existing tokens
    Melt {
        /// Amount of tokens to melt
        #[arg(long)]
        amount: u64,

        /// Address to melt from
        #[arg(long)]
        from: String,
    },

    /// Swap tokens
    Swap {
        /// Amount of tokens to swap
        #[arg(long)]
        amount: u64,

        /// Token to swap from
        #[arg(long)]
        from_token: String,

        /// Token to swap to
        #[arg(long)]
        to_token: String,
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

    let mut node_client = NodeClient::connect(cli.node_url.clone()).await?;

    wallet::db::create_tables(&mut db_conn)?;
    let node_id = wallet::db::insert_node(&mut db_conn, &cli.node_url)?;
    wallet::refresh_node_keysets(&mut db_conn, &mut node_client, node_id).await?;

    match cli.command {
        Commands::Mint { amount, unit } => {
            info!("Requesting {} to mint {} {}", cli.node_url, amount, unit);
            // Add mint logic here
            let mint_quote_response = wallet::create_mint_quote(
                &mut db_conn,
                &mut node_client,
                STARKNET_METHOD.to_string(),
                amount,
                unit.clone(),
            )
            .await?;

            info!(
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
<<<<<<< HEAD
=======
                // println!("state: {:?}", state);
>>>>>>> 20488e9 (initial: listener)

                if state == MintQuoteState::MnqsPaid {
                    info!("On-chain deposit received");
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
            info!("Finished.");
        }
        Commands::Melt { amount, from } => {
            println!("Melting {} tokens from {}", amount, from);
            // Add melt logic here
        }
        Commands::Swap {
            amount,
            from_token,
            to_token,
        } => {
            println!(
                "Swapping {} tokens from {} to {}",
                amount, from_token, to_token
            );
            // Add swap logic here
        }
    }

    Ok(())
}
