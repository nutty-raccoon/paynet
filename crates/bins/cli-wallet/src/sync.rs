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
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?;
        println!("Syncing node {} ({}) mint quotes", node_id, node_url);

        let (mut node_client, _) = connect_to_node(pool.clone(), node_id).await?;
        sync_mint_quotes(&pool, &mut node_client, node_id, &pending_quotes).await?;
    }
    for (node_id, pending_quotes) in pending_melt_quotes {
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?;
        println!("Syncing node {} ({}) melt quotes", node_id, node_url);

        let (mut node_client, _) = connect_to_node(pool.clone(), node_id).await?;
        sync_melt_quotes(&pool, &mut node_client, &pending_quotes).await?;
    }

    // Sync pending WADs
    println!("Syncing pending WADs");
    sync_pending_wads(pool).await?;

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

async fn sync_melt_quote(
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
        wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(anyhow!("unknown node id: {}", node_id))?
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

async fn sync_pending_wads(pool: Pool<SqliteConnectionManager>) -> Result<()> {
    let pending_wads = {
        let db_conn = pool.get()?;
        wallet::db::wad::get_pending_wads(&db_conn)?
    };

    if pending_wads.is_empty() {
        println!("No pending WADs to sync");
        return Ok(());
    }

    println!("Found {} pending WADs", pending_wads.len());

    for wad_record in pending_wads {
        let result = sync_single_wad(pool.clone(), &wad_record).await;

        match result {
            Ok(new_status) => {
                if let Some(status) = new_status {
                    println!("WAD {} updated to status: {:?}", wad_record.id, status);
                }
            }
            Err(e) => {
                eprintln!("Failed to sync WAD {}: {}", wad_record.id, e);
                // Mark as failed
                let db_conn = pool.get()?;
                let _ = wallet::db::wad::update_wad_status(
                    &db_conn,
                    wad_record.id,
                    wallet::db::wad::WadStatus::Failed,
                );
            }
        }
    }

    Ok(())
}

async fn sync_single_wad(
    pool: Pool<SqliteConnectionManager>,
    wad_record: &wallet::db::wad::WadRecord,
) -> Result<Option<wallet::db::wad::WadStatus>> {
    use node_client::{CheckStateRequest, ProofState};

    // Get proof public keys for this WAD
    let proof_ys = {
        let db_conn = pool.get()?;
        wallet::db::wad::get_wad_proofs(&db_conn, wad_record.id)?
    };

    if proof_ys.is_empty() {
        tracing::warn!("Empty WAD found (ID: {}), this should not occur", wad_record.id);
        return Ok(None);
    }

    // Parse the WAD data to get node information
    let compact_wad: wallet::types::compact_wad::CompactWad<starknet_types::Unit> =
        serde_json::from_str(&wad_record.wad_data)?;

    // Connect to the node
    let mut node_client = wallet::connect_to_node(&compact_wad.node_url).await?;

    // Check proof states using NUT-07
    let check_request = CheckStateRequest {
        ys: proof_ys.iter().map(|y| y.to_bytes().to_vec()).collect(),
    };

    let response = node_client.check_state(check_request).await?;
    let states = response.into_inner().states;

    // Analyze proof states to determine WAD status
    let mut all_spent = true;

    for state in states {
        match ProofState::try_from(state.state)? {
            ProofState::PsUnspent | ProofState::PsPending => {
                all_spent = false;
            }
            ProofState::PsSpent => {
                // Nothing to do - this maintains all_spent = true if all others are spent
            }
            _ => {
                return Err(anyhow!("Unexpected proof state encountered for WAD {}: {:?}", wad_record.id, state.state));
            }
        }
    }

    // Determine new status based on proof states
    let new_status = match wad_record.wad_type {
        wallet::db::wad::WadType::OUT => {
            // For outgoing WADs, finished when all proofs are spent
            if all_spent {
                tracing::info!("WAD {} all proofs spent, marking as Finished", wad_record.id);
                Some(wallet::db::wad::WadStatus::Finished)
            } else {
                tracing::info!("WAD {} some proofs not spent yet", wad_record.id);
                None
            }
        }
        wallet::db::wad::WadType::IN => {
            // For incoming WADs, finished when all proofs are received (spent in our wallet)
            if all_spent {
                tracing::info!("WAD {} all proofs received, marking as Finished", wad_record.id);
                Some(wallet::db::wad::WadStatus::Finished)
            } else {
                tracing::info!("WAD {} not all proofs received yet", wad_record.id);
                None
            }
        }
    };

    // Update status if changed
    if let Some(status) = &new_status {
        let db_conn = pool.get()?;
        wallet::db::wad::update_wad_status(&db_conn, wad_record.id, status.clone())?;
    }

    Ok(new_status)
}
