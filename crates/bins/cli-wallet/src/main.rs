use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueHint};
use node_client::NodeClient;
use nuts::Amount;
use primitive_types::U256;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use starknet_types::{Asset, STARKNET_STR, Unit, is_valid_starknet_address};
use starknet_types_core::felt::Felt;
use std::{fs, path::PathBuf, str::FromStr};
use sync::display_paid_melt_quote;
use tracing_subscriber::EnvFilter;
use wallet::{
    db::balance::Balance,
    melt::wait_for_payment,
    types::{
        NodeUrl, ProofState, Wad,
        compact_wad::{CompactWad, CompactWads},
    },
};

mod init;
mod sync;

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
        #[arg(long, short)]
        restore: Option<bool>,
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
        /// Ids of the nodes to use in priority
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
        #[arg(long, num_args = 1..,)]
        node_ids: Vec<u32>,
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
    /// Sync all pending operations
    #[command(
        about = "Sync all pending mint and melt operations",
        long_about = "Check all nodes for pending mint and melt quote updates and process them accordingly"
    )]
    Sync,
    #[command(
        about = "Generate a new wallet",
        long_about = "Generate a new wallet. This will create a new wallet with a new seed phrase and private key."
    )]
    Init,
    #[command(
        about = "Restore a wallet",
        long_about = "Restore a wallet. This will restore a wallet from a seed phrase and private key."
    )]
    Restore {
        /// The seed phrase
        #[arg(long, short)]
        seed_phrase: String,
    },
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
    fn read_wads(&self) -> Result<Vec<CompactWad<Unit>>> {
        let wad_string = if let Some(json_string) = &self.opt_wad_string {
            Ok(json_string.clone())
        } else if let Some(file_path) = &self.opt_wad_file_path {
            fs::read_to_string(file_path).map_err(|e| anyhow!("Failed to read wad file: {}", e))
        } else {
            Err(anyhow!("cli rules guarantee one and only one will be set"))
        }?;
        let wads: CompactWads<Unit> = wad_string.parse()?;

        Ok(wads.0)
    }
}

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

    let manager = SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::new(manager)?;
    let mut db_conn = pool.get()?;

    wallet::db::create_tables(&mut db_conn)?;

    let wallet_count = wallet::db::wallet::count_wallets(&db_conn)?;

    match cli.command {
        Commands::Init | Commands::Restore { .. } => {
            if wallet_count > 0 {
                println!("Wallet already exists");
                return Ok(());
            }
        }
        _ => {
            if wallet_count != 1 {
                println!("Wallet is not initialized. Run `init` or `restore` first");
                return Ok(());
            }
        }
    }

    match cli.command {
        Commands::Node(NodeCommands::Add { node_url, restore }) => {
            let node_url = wallet::types::NodeUrl::from_str(&node_url)?;

            let tx = db_conn.transaction()?;
            let (node_client, node_id) = wallet::register_node(pool.clone(), &node_url).await?;
            tx.commit()?;

            println!(
                "Successfully registered {} as node with id `{}`",
                &node_url, node_id
            );

            let wallet = wallet::db::wallet::get(&db_conn)?.unwrap();
            let should_restore = match restore {
                Some(true) => true,
                Some(false) => false,
                None => wallet.is_restored,
            };
            if should_restore {
                println!("Restoring proofs");
                wallet::node::restore(pool, node_id, node_client, wallet.private_key).await?;
                println!("Restoring done.");

                let balances = wallet::db::balance::get_for_node(&db_conn, node_id)?;
                println!("Balance for node {}:", node_id);
                for Balance { unit, amount } in balances {
                    println!("  {} {}", amount, unit);
                }
            }
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
                for Balance { unit, amount } in balances {
                    println!("  {} {}", amount, unit);
                }
            }
            None => {
                let nodes_with_balances = wallet::db::balance::get_for_all_nodes(&db_conn)?;
                for node_balances in nodes_with_balances {
                    println!(
                        "Balance for node {} ({}):",
                        node_balances.id, node_balances.url
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

            let amount = amount
                .checked_mul(asset.scale_factor())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let (amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

            let mint_quote_response = wallet::mint::create_quote(
                pool.clone(),
                &mut node_client,
                node_id,
                STARKNET_STR.to_string(),
                amount,
                unit,
            )
            .await?;

            println!(
                "MintQuote created with id: {}\nProceed to payment:\n{}",
                &mint_quote_response.quote, &mint_quote_response.request
            );

            match wallet::mint::wait_for_quote_payment(
                &db_conn,
                &mut node_client,
                STARKNET_STR.to_string(),
                mint_quote_response.quote.clone(),
            )
            .await?
            {
                wallet::mint::QuotePaymentIssue::Expired => {
                    println!("quote {} has expired", mint_quote_response.quote)
                }
                wallet::mint::QuotePaymentIssue::Paid => println!("On-chain deposit received"),
            }

            wallet::mint::redeem_quote(
                pool.clone(),
                &mut node_client,
                STARKNET_STR.to_string(),
                mint_quote_response.quote,
                node_id,
                unit,
                amount,
            )
            .await?;

            // TODO: remove mint_quote
            println!("Token stored. Finished.");
        }
        Commands::Melt {
            amount,
            asset,
            node_id,
            to,
        } => {
            let (mut node_client, _node_url) = connect_to_node(&mut db_conn, node_id).await?;

            println!("Melting {} {} tokens", amount, asset);

            // Convert user inputs to actionable types
            let on_chain_amount = amount
                .checked_mul(asset.scale_factor())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let unit = asset.find_best_unit();

            let payee_address = Felt::from_hex(&to)?;
            if !is_valid_starknet_address(&payee_address) {
                return Err(anyhow!("Invalid starknet address: {}", payee_address));
            }
            let method = STARKNET_STR.to_string();

            // Format starknet request
            let request = serde_json::to_string(&starknet_liquidity_source::MeltPaymentRequest {
                payee: payee_address,
                asset: starknet_types::Asset::Strk,
                amount: on_chain_amount.into(),
            })?;

            // Create the quote
            let melt_quote_response = wallet::melt::create_quote(
                pool.clone(),
                &mut node_client,
                node_id,
                method.clone(),
                unit,
                request,
            )
            .await?;
            println!("Melt quote created!");

            let melt_response = wallet::melt::pay_quote(
                pool.clone(),
                &mut node_client,
                node_id,
                melt_quote_response.quote.clone(),
                Amount::from(melt_quote_response.amount),
                method.clone(),
                unit,
            )
            .await?;
            println!("Melt submited!");

            if melt_response.state == node_client::MeltQuoteState::MlqsPaid as i32 {
                display_paid_melt_quote(melt_quote_response.quote, melt_response.transfer_ids);
            } else {
                match wait_for_payment(
                    pool.clone(),
                    &mut node_client,
                    method,
                    melt_quote_response.quote.clone(),
                )
                .await?
                {
                    Some(transfer_ids) => {
                        display_paid_melt_quote(melt_quote_response.quote, transfer_ids)
                    }
                    None => println!("Melt quote {} has expired", melt_quote_response.quote),
                }
            }
        }
        Commands::Send {
            amount,
            asset,
            node_ids,
            memo,
            output,
        } => {
            let output = output
                .map(|output_path| {
                    if output_path
                        .extension()
                        .ok_or_else(|| anyhow!("output file must have a .wad extension."))?
                        == "wad"
                    {
                        let output_path_string = output_path
                            .as_path()
                            .to_str()
                            .ok_or_else(|| anyhow!("invalid db path"))?
                            .to_string();

                        Ok((output_path, output_path_string))
                    } else {
                        Err(anyhow!("Output file should be a `.wad` file"))
                    }
                })
                .transpose()?;

            let amount = amount
                .checked_mul(asset.scale_factor())
                .ok_or(anyhow!("amount greater than the maximum for this asset"))?;
            let (total_amount, unit, _remainder) = asset.convert_to_amount_and_unit(amount)?;

            let node_ids_with_amount_to_use =
                wallet::send::plan_spending(&db_conn, total_amount, unit, &node_ids)?;

            let mut node_and_proofs = Vec::with_capacity(node_ids_with_amount_to_use.len());
            for (node_id, amount_to_use) in node_ids_with_amount_to_use {
                let (mut node_client, node_url) = connect_to_node(&mut db_conn, node_id).await?;

                let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
                    pool.clone(),
                    &mut node_client,
                    node_id,
                    amount_to_use,
                    unit,
                )
                .await?
                .ok_or(anyhow!("not enough funds"))?;

                println!(
                    "Spending {} {} from node {} ({})",
                    amount_to_use, asset, &node_id, &node_url
                );
                node_and_proofs.push((node_url, proofs_ids));
            }

            let mut wads = Vec::with_capacity(node_and_proofs.len());
            let mut should_revert = None;
            for (i, (node_url, proofs_ids)) in node_and_proofs.iter().enumerate() {
                let proofs = match wallet::load_tokens_from_db(&db_conn, proofs_ids) {
                    Ok(p) => p,
                    Err(e) => {
                        println!(
                            "Failed to load the following proofs for node {}: {}\nProof ids: {:?}\nReverting now.",
                            node_url, e, proofs_ids
                        );
                        should_revert = Some(i);
                        break;
                    }
                };

                let wad =
                    wallet::create_wad_from_proofs(node_url.clone(), unit, memo.clone(), proofs);
                wads.push(wad);
            }
            if let Some(max_reached) = should_revert {
                node_and_proofs
                    .iter()
                    .map(|(_, pids)| pids)
                    .take(max_reached)
                    .for_each(|proofs_id| {
                        if let Err(e) = wallet::db::proof::set_proofs_to_state(
                            &db_conn,
                            proofs_id,
                            ProofState::Unspent,
                        ) {
                            println!(
                                "failed to revet state of the following proofs: {}\nProofs ids: {:?}",
                                e, proofs_id
                            );
                        }
                    });

                return Err(anyhow!("wad creation reverted"));
            };

            let wads = CompactWads::new(wads);

            match output {
                Some((output_path, path_str)) => {
                    fs::write(&output_path, wads.to_string())
                        .map_err(|e| anyhow!("could not write to file {}: {}", path_str, e))?;
                    println!("Wad saved to {:?}", path_str);
                }
                None => {
                    println!("Wad:\n{}", wads);
                }
            }
        }
        Commands::Receive(WadArgs {
            opt_wad_string,
            opt_wad_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_string,
                opt_wad_file_path,
            };
            let wads = args.read_wads()?;

            for wad in wads {
                let (mut node_client, node_id) =
                    wallet::register_node(pool.clone(), &wad.node_url).await?;
                let CompactWad {
                    node_url,
                    unit,
                    memo,
                    proofs,
                } = wad;

                match wallet::receive_wad(pool.clone(), &mut node_client, node_id, wad.unit, proofs)
                    .await
                {
                    Ok(a) => {
                        println!("Received tokens on node `{}`", node_id);
                        if let Some(memo) = memo {
                            println!("Memo: {}", memo);
                        }
                        println!("{} {}", a, unit.as_str());
                    }
                    Err(e) => {
                        println!(
                            "failed to receive_wad from node {} ({}): {}",
                            node_id, node_url, e
                        );
                        continue;
                    }
                };
            }
        }
        Commands::DecodeWad(WadArgs {
            opt_wad_string,
            opt_wad_file_path,
        }) => {
            let args = WadArgs {
                opt_wad_string,
                opt_wad_file_path,
            };
            let wads = args.read_wads()?;

            for wad in wads {
                let regular_wad = Wad {
                    node_url: wad.node_url.clone(),
                    proofs: wad.proofs(),
                };

                println!("Node URL: {}", wad.node_url);
                if let Some(memo) = wad.memo() {
                    println!("Memo: {}", memo);
                }
                match wad.value() {
                    Ok(v) => println!("Total Value: {} {}", v, wad.unit()),
                    Err(_) => {
                        println!("sum of all proofs in the wad overflowed");
                        continue;
                    }
                };
                println!("\nDetailed Contents:");
                println!("{}", serde_json::to_string_pretty(&regular_wad)?);
            }
        }
        Commands::Sync => {
            sync::sync_all_pending_operations(pool).await?;
        }
        Commands::Init => {
            init::init(&db_conn)?;
        }
        Commands::Restore { seed_phrase } => {
            init::restore(&db_conn, seed_phrase)?;
        }
    }

    Ok(())
}

pub async fn connect_to_node(
    conn: &mut Connection,
    node_id: u32,
) -> Result<(NodeClient<tonic::transport::Channel>, NodeUrl)> {
    let node_url = wallet::db::node::get_url_by_id(conn, node_id)?
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
