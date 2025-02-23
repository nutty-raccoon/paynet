use anyhow::{Result, anyhow};
use node::{MintQuoteState, NodeClient};
use std::path::PathBuf;
use wallet::types::PreMint;

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
    let cli = Cli::parse();
    let db_path = cli
        .db_path
        .or(dirs::data_dir().map(|mut dp| {
            dp.push("cli-wallet.sqlite3");
            dp
        }))
        .ok_or(anyhow!("couldn't find `data_dir` on this computer"))?;
    println!("database located at `{:?}`", db_path);

    let mut db_conn = rusqlite::Connection::open(db_path)?;

    let mut node_client = NodeClient::connect(cli.node_url.clone()).await?;

    println!("0");
    wallet::db::create_tables(&mut db_conn)?;
    println!("1");
    wallet::db::insert_node(&mut db_conn, &cli.node_url)?;
    println!("2");
    wallet::refresh_node_keysets(&mut db_conn, &mut node_client, &cli.node_url).await?;
    println!("2");

    match cli.command {
        Commands::Mint { amount, unit } => {
            println!("Asking {} to mint {} {}", cli.node_url, amount, unit);
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
                "MintQuote created with id: {}\nProceed to payment:\n{:?}",
                &mint_quote_response.quote, &mint_quote_response.request
            );

            loop {
                let state = wallet::get_mint_quote_state(
                    &mut db_conn,
                    &mut node_client,
                    STARKNET_METHOD.to_string(),
                    mint_quote_response.quote.clone(),
                )
                .await?;
                println!("state: {:?}", state);

                if state == MintQuoteState::MnqsPaid {
                    break;
                }
            }

            let keyset_id = wallet::get_active_keyst_for_unit(&mut db_conn, &cli.node_url, unit)?;

            let pre_mints = PreMint::generate_for_amount(amount.into(), keyset_id)?;

            let keyset_id_as_vec = keyset_id.to_bytes().to_vec();

            let outputs = pre_mints
                .iter()
                .map(|pm| node::BlindedMessage {
                    amount: pm.blinded_message.amount.into(),
                    keyset_id: keyset_id_as_vec.clone(),
                    blinded_secret: pm.blinded_message.blinded_secret.to_bytes().to_vec(),
                })
                .collect();

            let mint_response = node_client
                .mint(node::MintRequest {
                    method: STARKNET_METHOD.to_string(),
                    quote: mint_quote_response.quote,
                    outputs,
                })
                .await?
                .into_inner();

            println!("{:?}", mint_response);
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
