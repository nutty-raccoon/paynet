use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueHint};
use itertools::Itertools;
use node::{MintQuoteState, NodeClient, QuoteStateRequest, hash_melt_request};
use nuts::Amount;
use primitive_types::U256;
use rusqlite::Connection;
use starknet_types::{Asset, Unit, is_valid_starknet_address};
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
        /// Polling interval in seconds (default: 1)
        #[arg(long, default_value = "1")]
        poll_interval: u64,
        /// Timeout for melt operation in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,
    },
    /// Sync all pending operations
    #[command(
        about = "Sync all pending operations",
        long_about = "Sync all pending mint and melt operations. Inspect the database for pending quotes and ask the nodes about updates. Finalize operations if possible."
    )]
    Sync {
        /// Polling interval in seconds (default: 1)
        #[arg(long, default_value = "1")]
        poll_interval: u64,
        /// Timeout for sync operations in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,
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
    #[arg(long = "string", short = 's', value_name = "WAD STRING")]
    opt_wad_string: Option<String>,
    #[arg(long = "file", short = 'f', value_name = "PATH", value_hint = ValueHint::FilePath)]
    opt_wad_file_path: Option<String>,
}

impl WadArgs {
    fn read_wad(&self) -> Result<CompactWad<Unit>> {
        let wad_string = if let Some(json_string) = &self.opt_wad_string {
            Ok(json_string.clone())
        } else if let Some(file_path) = &self.opt_wad_file_path {
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

                let state = match wallet::get_mint_quote_state(
                    &db_conn,
                    &mut node_client,
                    STARKNET_METHOD.to_string(),
                    mint_quote_response.quote.clone(),
                )
                .await?
                {
                    Some(new_state) => new_state,
                    None => {
                        println!("quote {} has expired", mint_quote_response.quote);
                        return Ok(());
                    }
                };

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
        Commands::Sync {
            poll_interval,
            timeout,
        } => {
            sync_all_pending_operations(&mut db_conn, poll_interval, timeout).await?;
        }
        Commands::Melt {
            amount,
            asset,
            node_id,
            to,
            poll_interval,
            timeout,
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

            let payee_address = Felt::from_hex(&to)?;
            if !is_valid_starknet_address(&payee_address) {
                return Err(anyhow!("Invalid starknet address: {}", payee_address));
            }

            let melt_request = node::MeltRequest {
                method: STARKNET_METHOD.to_string(),
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

            acknowledge(
                &mut node_client,
                nuts::nut19::Route::Melt,
                melt_request_hash,
            )
            .await?;

            // Use timeout for the entire melt polling operation
            let melt_result = tokio::time::timeout(Duration::from_secs(timeout), async {
                loop {
                    let melt_quote_state_response = node_client
                        .melt_quote_state(QuoteStateRequest {
                            method: STARKNET_METHOD.to_string(),
                            quote: resp.quote.clone(),
                        })
                        .await?
                        .into_inner();

                    let melt_state = node::MeltState::try_from(melt_quote_state_response.state)
                        .map_err(|_| anyhow!("Invalid melt state received"))?;

                    match melt_state {
                        node::MeltState::MlqsPaid => {
                            // Update database state
                            wallet::db::update_melt_quote_state(
                                &db_conn,
                                &resp.quote,
                                melt_state as i32,
                            )?;

                            println!(
                                "{}",
                                format_melt_transfers_id_into_term_message(
                                    melt_quote_state_response.transfer_ids
                                )
                            );
                            break;
                        }
                        node::MeltState::MlqsUnpaid => {
                            // Check if expired
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            if now >= melt_quote_state_response.expiry {
                                wallet::db::delete_expired_melt_quote(&db_conn, &resp.quote)?;
                                println!("Melt quote expired and removed");
                                break;
                            }
                            // Continue polling with configurable interval
                            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
                        }
                        node::MeltState::MlqsPending => {
                            // Continue polling with configurable interval
                            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
                        }
                        _ => {
                            // Handle unknown states - continue polling with configurable interval
                            tokio::time::sleep(Duration::from_secs(poll_interval)).await;
                        }
                    }
                }
                Ok::<(), anyhow::Error>(())
            })
            .await;

            match melt_result {
                Ok(Ok(())) => {
                    // Melt completed successfully
                }
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    return Err(anyhow!(
                        "Melt operation timed out after {} seconds",
                        timeout
                    ));
                }
            }
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
                        .ok_or_else(|| anyhow!("output file must have a .wad extension."))?
                        == "wad"
                    {
                        Ok(output_path)
                    } else {
                        Err(anyhow!("Output file should be a `.wad` file"))
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
            opt_wad_string,
            opt_wad_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_string,
                opt_wad_file_path,
            };
            let wad = args.read_wad()?;

            let (mut node_client, node_id) =
                wallet::register_node(&db_conn, wad.node_url.clone()).await?;
            println!("Receiving tokens on node `{}`", node_id);
            if let Some(memo) = wad.memo() {
                println!("Memo: {}", memo);
            }

            let amount_received = wallet::receive_wad(
                &mut db_conn,
                &mut node_client,
                node_id,
                wad.unit.as_str(),
                &wad.proofs,
            )
            .await?;

            println!("Received:");
            println!("{} {}", amount_received, wad.unit.as_str());
        }
        Commands::DecodeWad(WadArgs {
            opt_wad_string,
            opt_wad_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_string,
                opt_wad_file_path,
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

async fn sync_all_pending_operations(
    db_conn: &mut Connection,
    poll_interval: u64,
    timeout: u64,
) -> Result<()> {
    println!("Starting sync of all pending operations...");
    println!(
        "Using poll interval: {}s, timeout: {}s",
        poll_interval, timeout
    );

    // Get all nodes
    let nodes = wallet::db::node::fetch_all(db_conn)?;

    for (node_id, node_url) in nodes {
        println!("Syncing operations for node {} ({})", node_id, node_url);

        let (mut node_client, _) = connect_to_node(db_conn, node_id).await?;

        // Set timeout for node client operations
        let sync_result = tokio::time::timeout(Duration::from_secs(timeout), async {
            // Sync mint quotes
            sync_mint_quotes_for_node(db_conn, &mut node_client, node_id, poll_interval).await?;

            // Sync melt quotes
            sync_melt_quotes_for_node(db_conn, &mut node_client, node_id, poll_interval).await?;

            Ok::<(), anyhow::Error>(())
        })
        .await;

        match sync_result {
            Ok(Ok(())) => {
                println!("Successfully synced node {}", node_id);
            }
            Ok(Err(e)) => {
                println!("Error syncing node {}: {}", node_id, e);
                // Continue with other nodes
            }
            Err(_) => {
                println!("Timeout syncing node {} after {}s", node_id, timeout);
                // Continue with other nodes
            }
        }
    }

    println!("Sync completed.");
    Ok(())
}

async fn sync_mint_quotes_for_node(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<tonic::transport::Channel>,
    node_id: u32,
    poll_interval: u64,
) -> Result<()> {
    let pending_quotes = wallet::db::get_pending_mint_quotes(db_conn)?;

    // Find quotes for this node
    let node_quotes = pending_quotes
        .into_iter()
        .find(|(id, _)| *id == node_id)
        .map(|(_, quotes)| quotes)
        .unwrap_or_default();

    let quotes_count = node_quotes.len();

    for (method, quote_id, previous_state, unit, amount) in node_quotes {
        let new_state =
            match wallet::get_mint_quote_state(db_conn, node_client, method, quote_id.clone())
                .await?
            {
                Some(new_state) => new_state,
                None => {
                    println!("Mint quote {} has expired", quote_id);
                    continue;
                }
            };

        let previous_state = MintQuoteState::try_from(previous_state).unwrap();
        if previous_state == MintQuoteState::MnqsUnpaid && new_state == MintQuoteState::MnqsPaid {
            println!("On-chain deposit received for mint quote {}", quote_id);
            let tx = db_conn.transaction()?;
            wallet::mint_and_store_new_tokens(
                &tx,
                node_client,
                STARKNET_METHOD.to_string(),
                quote_id,
                node_id,
                unit.as_str(),
                Amount::from(amount),
            )
            .await?;
            tx.commit()?;
            println!("Tokens stored for mint quote.");
        }

        // Small delay between quote processing to avoid overwhelming the node
        if quotes_count > 0 {
            tokio::time::sleep(Duration::from_millis(poll_interval * 100)).await;
        }
    }

    Ok(())
}

async fn sync_melt_quotes_for_node(
    db_conn: &mut Connection,
    node_client: &mut NodeClient<tonic::transport::Channel>,
    node_id: u32,
    poll_interval: u64,
) -> Result<()> {
    let pending_quotes = wallet::db::get_pending_melt_quotes(db_conn)?;

    // Find quotes for this node
    let node_quotes = pending_quotes
        .into_iter()
        .find(|(id, _)| *id == node_id)
        .map(|(_, quotes)| quotes)
        .unwrap_or_default();

    let quotes_count = node_quotes.len();

    for (quote_id, previous_state, expiry) in node_quotes {
        let new_state = match wallet::get_melt_quote_state(
            db_conn,
            node_client,
            STARKNET_METHOD.to_string(),
            quote_id.clone(),
        )
        .await?
        {
            Some(new_state) => new_state,
            None => {
                println!("Melt quote {} has expired", quote_id);
                continue;
            }
        };

        let previous_state_enum = match previous_state {
            1 => node::MeltState::MlqsUnpaid,
            2 => node::MeltState::MlqsPending,
            3 => node::MeltState::MlqsPaid,
            _ => continue, // Skip unknown states
        };

        match new_state {
            node::MeltState::MlqsPaid if previous_state_enum != node::MeltState::MlqsPaid => {
                println!("Melt quote {} completed successfully", quote_id);
                wallet::db::update_melt_quote_state(db_conn, &quote_id, new_state as i32)?;
            }
            node::MeltState::MlqsUnpaid => {
                // Check if expired
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now >= expiry {
                    println!("Melt quote {} expired and will be removed", quote_id);
                    wallet::db::delete_expired_melt_quote(db_conn, &quote_id)?;
                }
            }
            _ => {
                // Update state in database for pending or other states
                wallet::db::update_melt_quote_state(db_conn, &quote_id, new_state as i32)?;
            }
        }

        // Small delay between quote processing to avoid overwhelming the node
        if quotes_count > 0 {
            tokio::time::sleep(Duration::from_millis(poll_interval * 100)).await;
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
