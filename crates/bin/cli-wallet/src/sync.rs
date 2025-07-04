use anyhow::{Result, anyhow};
use node_client::NodeClient;
use nuts::nut04::MintQuoteState;
use nuts::nut05::MeltQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tonic::transport::Channel;
use wallet::db::melt_quote::PendingMeltQuote;
use wallet::db::mint_quote::PendingMintQuote;
use wallet::types::NodeUrl;

const STARKNET_STR: &str = "starknet";

pub async fn sync_all_pending_operations(pool: Pool<SqliteConnectionManager>) -> Result<()> {
    let db_conn = pool.get()?;
    let (pending_mint_quotes, pending_melt_quotes) = {
        let mint_quotes = wallet::db::mint_quote::get_pendings(&db_conn)?;
        let melt_quotes = wallet::db::melt_quote::get_pendings(&db_conn)?;
        (mint_quotes, melt_quotes)
    };

    for (node_id, pending_quotes) in pending_mint_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?;
        println!("Syncing node {} ({}) mint quotes", node_id, node_url);

        let (mut node_client, _) = connect_to_node(pool.clone(), node_id).await?;
        sync_mint_quotes(&pool, &mut node_client, node_id, &pending_quotes).await?;
    }
    for (node_id, pending_quotes) in pending_melt_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?;
        println!("Syncing node {} ({}) melt quotes", node_id, node_url);

        let (mut node_client, _) = connect_to_node(pool.clone(), node_id).await?;
        sync_melt_quotes(&pool, &mut node_client, &pending_quotes).await?;
    }

    println!("Sync completed for all nodes");
    Ok(())
}

async fn sync_mint_quotes(
    pool: &Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    pending_mint_quotes: &[PendingMintQuote],
) -> Result<()> {
    println!("pending: {:?}", pending_mint_quotes);
    for pending_mint_quote in pending_mint_quotes {
        let new_state = {
            let db_conn = pool.get()?;

            match wallet::mint::get_quote_state(
                &db_conn,
                node_client,
                pending_mint_quote.method.clone(),
                pending_mint_quote.id.clone(),
            )
            .await?
            {
                Some(new_state) => new_state,
                None => {
                    println!("Mint quote {} has expired", pending_mint_quote.id);
                    continue;
                }
            }
        };

        if new_state == MintQuoteState::Paid {
            println!(
                "On-chain deposit received for mint quote {}",
                pending_mint_quote.id
            );

            // Redeem the quote
            if let Err(e) = wallet::mint::redeem_quote(
                pool.clone(),
                node_client,
                STARKNET_STR.to_string(),
                pending_mint_quote.id.clone(),
                node_id,
                &pending_mint_quote.unit,
                pending_mint_quote.amount,
            )
            .await
            {
                eprintln!(
                    "Failed to redeem mint quote {}: {}",
                    pending_mint_quote.id, e
                );
            } else {
                println!("Successfully redeemed mint quote {}", pending_mint_quote.id);
            }
        }
    }

    Ok(())
}

async fn sync_melt_quotes(
    pool: &Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    pending_melt_quotes: &[PendingMeltQuote],
) -> Result<()> {
    for pending_melt_quote in pending_melt_quotes {
        sync_melt_quote(
            pool.clone(),
            node_client,
            STARKNET_STR.to_string(),
            pending_melt_quote.id.clone(),
        )
        .await?;
    }

    Ok(())
}

pub async fn sync_melt_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut NodeClient<Channel>,
    method: String,
    quote_id: String,
) -> Result<bool> {
    let melt_quote = wallet::sync::melt_quote(pool, node_client, method, quote_id.clone()).await?;

    let is_final = match melt_quote {
        Some((MeltQuoteState::Paid, tx_ids)) => {
            display_paid_melt_quote(quote_id, tx_ids);
            true
        }
        None => {
            println!("Melt quote {} has expired", quote_id);
            true
        }
        _ => false,
    };

    Ok(is_final)
}

pub fn display_paid_melt_quote(quote_id: String, tx_ids: Vec<String>) {
    println!("Melt quote {} completed successfully", quote_id);
    if !tx_ids.is_empty() {
        println!(
            "tx hashes: {}",
            format_melt_transfers_id_into_term_message(tx_ids)
        );
    }
}

async fn connect_to_node(
    pool: Pool<SqliteConnectionManager>,
    node_id: u32,
) -> Result<(NodeClient<Channel>, NodeUrl)> {
    let node_url = {
        let db_conn = pool.get()?;
        wallet::db::get_node_url(&db_conn, node_id)?
            .ok_or_else(|| anyhow!("Node {} not found", node_id))?
    };

    let node_client = wallet::connect_to_node(&node_url)
        .await
        .map_err(|e| anyhow!("Failed to connect to node {}: {}", node_url, e))?;

    Ok((node_client, node_url))
}

pub fn format_melt_transfers_id_into_term_message(transfer_ids: Vec<String>) -> String {
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
