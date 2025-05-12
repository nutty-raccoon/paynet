use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueHint};
use itertools::Itertools;
use node::{MintQuoteState, NodeClient, hash_melt_request};
use nuts::Amount;
use primitive_types::U256;
use rusqlite::Connection;
use starknet_types::{Asset, Unit};
use starknet_types_core::felt::Felt;
use std::{fs, path::PathBuf, str::FromStr, time::Duration};
use tracing_subscriber::EnvFilter;
use wallet::{
    acknowledge,
    types::compact_wad::{CompactKeysetProofs, CompactProof, CompactWad},
    types::{NodeUrl, Wad},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// The path to the wallet sqlite database
    ///
    /// If left blank the default one will be used:
    /// `dirs::data_dir().cli-wallet.sqlite3`
    #[arg(long, value_hint(ValueHint::FilePath))]
    db_path: Option<PathBuf>,
}

#[derive(Subcommand)]
enum MintCommands {
    /// Mint new tokens
    #[command(
        about = "Mint some tokens",
        long_about = "Mint some tokens. Will require you to send some assets to the node."
    )]
    New {
        /// Amount requested
        #[arg(long, value_parser = parse_asset_amount)]
        amount: U256,
        /// Asset requested
        #[arg(long, value_parser = Asset::from_str)]
        asset: Asset,
        /// Id of the node to use
        #[arg(long)]
        node_id: u32,
    },

    /// Sync
    #[command(
        about = "Sync ongoing mint operation",
        long_about = "Sync ongoing mint operation. Inspect the database for pending quote and ask the node about updates. Finalize the mint if possible."
    )]
    Sync {},
}

#[derive(Subcommand)]
enum NodeCommands {
    /// Register a new node
    #[command(
        about = "Register a new node",
        long_about = "Register a new node. Each one is given an unique incremental integer value as id."
    )]
    Add {
        /// Url of the node
        #[arg(long, short)]
        node_url: String,
    },
    /// List all know nodes
    #[command(
        about = "List all the registered nodes",
        long_about = "List all the registered nodes. Display their id and url."
    )]
    #[clap(name = "ls")]
    List {},
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Node(NodeCommands),
    /// Show balance
    #[command(
        about = "Display your balances accross all nodes",
        long_about = "Display your balances accross all nodes. For each node, show the total available amount for each unit."
    )]
    Balance {
        /// If specified, only show balance for this node
        #[arg(long, short)]
        node_id: Option<u32>,
    },
    #[command(subcommand)]
    Mint(MintCommands),
    /// Melt existing tokens
    #[command(
        about = "Melt some tokens",
        long_about = "Melt some tokens. Send them to the node and receive the original asset back."
    )]
    Melt {
        /// Amount to melt
        #[arg(long, value_parser = parse_asset_amount)]
        amount: U256,
        /// Unit to melt
        #[arg(long, value_parser = Asset::from_str)]
        asset: Asset,
        /// Id of the node to use
        #[arg(long)]
        node_id: u32,
        #[arg(long)]
        to: String,
    },
    /// Send tokens
    #[command(
        about = "Send some tokens",
        long_about = "Send some tokens. Store them in a wad, ready to be shared"
    )]
    Send {
        /// Amount to send
        #[arg(long, value_parser = parse_asset_amount)]
        amount: U256,
        /// Unit to send
        #[arg(long, value_parser = Asset::from_str)]
        asset: Asset,
        /// Id of the node to use
        #[arg(long)]
        node_id: u32,
        /// Optional memo to add context to the wad
        #[arg(long)]
        memo: Option<String>,
        /// File where to save the token wad        
        #[arg(long, short, value_hint(ValueHint::FilePath))]
        output: Option<PathBuf>,
    },
    /// Receive a wad of proofs
    #[command(
        about = "Receive a wad of tokens",
        long_about = "Receive a wad of tokens. Store them on them wallet for later use"
    )]
    Receive(WadArgs),
    /// Decode a wad to view its contents
    #[command(
        about = "Decode a wad to print its contents",
        long_about = "Decode a wad to print its contents in a friendly format"
    )]
    DecodeWad(WadArgs),
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct WadArgs {
    #[arg(long = "string", short = 's', value_name = "JSON STRING")]
    opt_wad_json_string: Option<String>,
    #[arg(long = "file", short = 'f', value_name = "PATH", value_hint = ValueHint::FilePath)]
    opt_wad_json_file_path: Option<String>,
}

impl WadArgs {
    fn read_wad(&self) -> Result<CompactWad<Unit>> {
        let wad_string = if let Some(json_string) = &self.opt_wad_json_string {
            Ok(json_string.clone())
        } else if let Some(file_path) = &self.opt_wad_json_file_path {
            fs::read_to_string(file_path).map_err(|e| anyhow!("Failed to read wad file: {}", e))
        } else {
            Err(anyhow!("cli rules guarantee one and only one will be set"))
        }?;
        let wad: CompactWad<Unit> = wad_string.parse()?;

        Ok(wad)
    }
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
    println!(
        "Using database at {:?}\n",
        db_path
            .as_path()
            .to_str()
            .ok_or(anyhow!("invalid db path"))?
    );

    let mut db_conn = rusqlite::Connection::open(db_path)?;

    wallet::db::create_tables(&mut db_conn)?;

    match cli.command {
        Commands::Node(NodeCommands::Add { node_url }) => {
            let node_url = wallet::types::NodeUrl::from_str(&node_url)?;

            let tx = db_conn.transaction()?;
            let (mut _node_client, node_id) = wallet::register_node(&tx, node_url.clone()).await?;
            tx.commit()?;
            println!(
                "Successfully registered {} as node with id `{}`",
                &node_url, node_id
            );
        }
        Commands::Node(NodeCommands::List {}) => {
            let nodes = wallet::db::node::fetch_all(&db_conn)?;

            println!("Available nodes");
            for (id, url) in nodes {
                println!("{} {}", id, url);
            }
        }
        Commands::Balance { node_id } => match node_id {
            Some(node_id) => {
                let balances = wallet::db::balance::get_for_node(&db_conn, node_id)?;
                println!("Balance for node {}:", node_id);
                for (unit, amount) in balances {
                    println!("  {} {}", amount, unit);
                }
            }
            None => {
                let nodes_with_balances = wallet::db::balance::get_for_all_nodes(&db_conn)?;
                for node_balances in nodes_with_balances {
                    println!(
                        "Balance for node {} ({}):",
                        node_balances.node_id, node_balances.url
                    );
                    for balance in node_balances.balances {
                        println!("  {} {}", balance.amount, balance.unit);
                    }
                }
            }
        },
        Commands::Mint(MintCommands::New {
            amount,
            asset,
            node_id,
        }) => {
            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Requesting {} to mint {} {}", &node_url, amount, asset);

            let tx = db_conn.transaction()?;

            let amount = amount
                .checked_mul(asset.precision())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

            let mint_quote_response = wallet::create_mint_quote(
                &tx,
                &mut node_client,
                node_id,
                STARKNET_METHOD.to_string(),
                amount,
                unit.as_str(),
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
                    &db_conn,
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
                unit.as_str(),
                amount,
            )
            .await?;
            tx.commit()?;

            // TODO: remove mint_quote
            println!("Token stored. Finished.");
        }
        Commands::Mint(MintCommands::Sync {}) => {
            let pending_quotes = wallet::db::get_pending_mint_quotes(&db_conn)?;
            for (node_id, quotes) in pending_quotes {
                let (mut node_client, _node_url) = connect_to_node(&mut db_conn, node_id).await?;
                for (method, quote_id, previous_state, unit, amount) in quotes {
                    let tx = db_conn.transaction()?;
                    let new_state = wallet::get_mint_quote_state(
                        &tx,
                        &mut node_client,
                        method,
                        quote_id.clone(),
                    )
                    .await?;

                    let previous_state = MintQuoteState::try_from(previous_state).unwrap();
                    if previous_state == MintQuoteState::MnqsUnpaid
                        && new_state == MintQuoteState::MnqsPaid
                    {
                        println!("On-chain deposit received for quote {}", quote_id);
                        wallet::mint_and_store_new_tokens(
                            &tx,
                            &mut node_client,
                            STARKNET_METHOD.to_string(),
                            quote_id,
                            node_id,
                            unit.as_str(),
                            Amount::from(amount),
                        )
                        .await?;
                        println!("Token stored.");
                    }
                    tx.commit()?;
                }
            }
        }
        Commands::Melt {
            amount,
            asset,
            node_id,
            to,
        } => {
            let (mut node_client, _node_url) = connect_to_node(&mut db_conn, node_id).await?;

            println!("Melting {} {} tokens", amount, asset);

            let amount = amount
                .checked_mul(asset.precision())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

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

            let melt_request = node::MeltRequest {
                method: STARKNET_METHOD.to_string(),
                unit: unit.to_string(),
                request: serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                    payee: Felt::from_hex(&to)?,
                    asset: starknet_types::Asset::Strk,
                })?,
                inputs: wallet::convert_inputs(&inputs),
            };
            let melt_request_hash = hash_melt_request(&melt_request);
            let resp = node_client.melt(melt_request).await?.into_inner();

            wallet::db::register_melt_quote(&tx, node_id, &resp)?;

            tx.commit()?;

            acknowledge(
                &mut node_client,
                nuts::nut19::Route::Melt,
                melt_request_hash,
            )
            .await?;

            let tx_hash = Felt::from_bytes_be_slice(&resp.transfer_id);
            println!("Melt done. Withdrawal settled with tx: {:#x}", tx_hash);
        }
        Commands::Send {
            amount,
            asset,
            node_id,
            memo,
            output,
        } => {
            let output: Option<PathBuf> = output
                .map(|output_path| {
                    if output_path
                        .extension()
                        .ok_or_else(|| anyhow!("output file must have a .json extension."))?
                        == "json"
                    {
                        Ok(output_path)
                    } else {
                        Err(anyhow!("Output file should be a `.json` file"))
                    }
                })
                .transpose()?;

            let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;
            println!("Sending {} {} using node {}", amount, asset, &node_url);

            let amount = amount
                .checked_mul(asset.precision())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

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

            match output {
                Some(output_path) => {
                    let path_str = output_path
                        .as_path()
                        .to_str()
                        .ok_or_else(|| anyhow!("invalid db path"))?;
                    fs::write(&output_path, wad.to_string())
                        .map_err(|e| anyhow!("could not write to file {}: {}", path_str, e))?;
                    println!("Wad saved to {:?}", path_str);
                }
                None => {
                    println!("Wad:\n{}", wad);
                }
            }
            tx.commit()?;
        }
        Commands::Receive(WadArgs {
            opt_wad_json_string,
            opt_wad_json_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_json_string,
                opt_wad_json_file_path,
            };
            let wad = args.read_wad()?;

            let (mut node_client, node_id) =
                wallet::register_node(&db_conn, wad.node_url.clone()).await?;
            println!("Receiving tokens on node `{}`", node_id);
            if let Some(memo) = wad.memo() {
                println!("Memo: {}", memo);
            }

            let tx = db_conn.transaction()?;
            let amounts_received_per_unit =
                wallet::receive_wad(&tx, &mut node_client, node_id, &wad.proofs()).await?;
            tx.commit()?;

            println!("Received:");
            for (unit, amount) in amounts_received_per_unit {
                println!("{} {}", amount, unit);
            }
        }
        Commands::DecodeWad(WadArgs {
            opt_wad_json_string,
            opt_wad_json_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_json_string,
                opt_wad_json_file_path,
            };
            let wad = args.read_wad()?;

            let regular_wad = Wad {
                node_url: wad.node_url.clone(),
                proofs: wad.proofs(),
            };

            println!("Node URL: {}", wad.node_url);
            println!("Unit: {}", wad.unit());
            if let Some(memo) = wad.memo() {
                println!("Memo: {}", memo);
            }
            println!("Total Value: {} {}", wad.value()?, wad.unit());
            println!("\nDetailed Contents:");
            println!("{}", serde_json::to_string_pretty(&regular_wad)?);
        }
    }

    Ok(())
}

pub async fn connect_to_node(
    conn: &mut Connection,
    node_id: u32,
) -> Result<(NodeClient<tonic::transport::Channel>, NodeUrl)> {
    let node_url = wallet::db::get_node_url(conn, node_id)?
        .ok_or_else(|| anyhow!("no node with id {node_id}"))?;
    let node_client = wallet::connect_to_node(&node_url).await?;
    Ok((node_client, node_url))
}

pub fn parse_asset_amount(amount: &str) -> Result<U256, std::io::Error> {
    if amount.starts_with("0x") || amount.starts_with("0X") {
        U256::from_str_radix(amount, 16)
    } else {
        U256::from_str_radix(amount, 10)
    }
    .map_err(std::io::Error::other)
}
