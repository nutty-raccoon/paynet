use std::str::FromStr;

use anyhow::Result;
use e2e_tests::{db_connection, read_env_variables};
use test_utils::e2e::starknet::wallet_ops::{WalletOps, recieve_already_spent_wad};
use wallet::types::NodeUrl;

#[tokio::test]
pub async fn run_e2e() -> Result<()> {
    let env = read_env_variables()?;
    let db_pool = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let mut node_client = wallet::connect_to_node(&node_url, None).await?;
    let node_id = wallet::node::register(db_pool.clone(), &mut node_client, &node_url).await?;
    let mut wallet_ops = WalletOps::new(db_pool.clone(), node_id, node_client);

    // Init
    let seed_phrase = wallet_ops.init()?;
    // Mint
    wallet_ops
        .mint(10.into(), starknet_types::Asset::Strk, env)
        .await?;
    // Send
    let wad = wallet_ops
        .send(
            node_id,
            node_url.clone(),
            10.into(),
            starknet_types::Asset::Strk,
            Some("Here come the money".to_string()),
        )
        .await?;
    let wad_record = wallet::db::wad::get_recent_wads(&*db_pool.get()?, 1)?[0].clone();
    assert_eq!(wad_record.r#type, wallet::db::wad::WadType::OUT);
    assert_eq!(wad_record.status, wallet::db::wad::WadStatus::Pending);
    assert_eq!(wad_record.node_url, node_url.to_string());
    // Recive
    wallet_ops.receive(&wad).await?;
    let wad_records = wallet::db::wad::get_recent_wads(&*db_pool.get()?, 2)?;
    assert_eq!(wad_records.len(), 2);
    assert_ne!(wad_records[0].r#type, wad_records[1].r#type);
    for record in wad_records {
        assert_eq!(record.id, wad_record.id);
        assert_eq!(record.status, wallet::db::wad::WadStatus::Finished);
        assert_eq!(record.node_url, node_url.to_string());
    }

    recieve_already_spent_wad(&mut wallet_ops, &wad).await?;

    // Melt
    wallet_ops
        .melt(
            5.into(),
            starknet_types::Asset::Strk,
            "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
        )
        .await?;
    // Restore
    let pre_restore_balances = wallet_ops.balance()?;
    assert!(!pre_restore_balances.is_empty());

    let env = read_env_variables()?;
    let db_pool = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;
    let mut node_client = wallet::connect_to_node(&node_url, None).await?;
    let node_id = wallet::node::register(db_pool.clone(), &mut node_client, &node_url).await?;
    let wallet_ops = WalletOps::new(db_pool.clone(), node_id, node_client);

    assert!(wallet_ops.balance()?.is_empty());
    wallet_ops.restore(seed_phrase).await?;
    let post_restore_balances = wallet_ops.balance()?;
    assert_eq!(pre_restore_balances, post_restore_balances);

    Ok(())
}
