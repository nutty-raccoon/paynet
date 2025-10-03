use anyhow::{Result, anyhow};
use cashu_client::GrpcClient;
use nuts::nut05::MeltQuoteState;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use wallet::ConnectToNodeResponse;
use wallet::db::melt_quote::PendingMeltQuote;
use wallet::melt::format_melt_transfers_id_into_term_message;

use crate::SEED_PHRASE_MANAGER;

const STARKNET_STR: &str = "starknet";

pub async fn sync_all_pending_operations(pool: Pool<SqliteConnectionManager>) -> Result<()> {
    let db_conn = pool.get()?;
    let (pending_mint_quotes, pending_melt_quotes) = {
        let mint_quotes = wallet::db::mint_quote::get_pendings(&db_conn)?;
        let melt_quotes = wallet::db::melt_quote::get_pendings(&db_conn)?;
        (mint_quotes, melt_quotes)
    };

    for (node_id, pending_quotes) in pending_mint_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?;
        println!("Syncing node {} ({}) mint quotes", node_id, node_url);

        let mut node_client = connect_to_node(pool.clone(), node_id).await?;
        wallet::sync::mint_quotes(
            SEED_PHRASE_MANAGER,
            pool.clone(),
            &mut node_client.client,
            node_id,
            pending_quotes,
            true,
        )
        .await?;
    }
    for (node_id, pending_quotes) in pending_melt_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?;
        println!("Syncing node {} ({}) melt quotes", node_id, node_url);

        let mut node_client = connect_to_node(pool.clone(), node_id).await?;
        sync_melt_quotes(&pool, &mut node_client.client, &pending_quotes).await?;
    }

    // Sync pending WADs using the lib wallet function i
    println!("Syncing pending WADs");
    let wad_results = wallet::sync::pending_wads(pool, None).await?;

    for result in wad_results {
        match result.result {
            // No status change
            Ok(None) => {}
            Ok(Some(status)) => println!("WAD {} updated to status: {:?}", result.wad_id, status),
            Err(e) => eprintln!("Failed to sync WAD {}: {}", result.wad_id, e),
        }
    }

    println!("Sync completed for all nodes");
    Ok(())
}

async fn sync_melt_quotes(
    pool: &Pool<SqliteConnectionManager>,
    node_client: &mut GrpcClient,
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

async fn sync_melt_quote(
    pool: Pool<SqliteConnectionManager>,
    node_client: &mut GrpcClient,
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
) -> Result<ConnectToNodeResponse<GrpcClient>> {
    let node_url = {
        let db_conn = pool.get()?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?
    };

    let client = wallet::connect_to_node(node_url, None)
        .await
        .map_err(|e| anyhow!("Failed to connect to node: {}", e))?;

    Ok(client)
}
