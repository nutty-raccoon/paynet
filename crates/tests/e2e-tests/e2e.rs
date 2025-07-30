use std::str::FromStr;

use anyhow::Result;
use e2e_tests::{db_connection, read_env_variables};
use test_utils::e2e::starknet::wallet_ops::WalletOps;
use wallet::types::{NodeUrl, compact_wad::DleqVerificationResult};

#[tokio::test]
pub async fn run_e2e() -> Result<()> {
    let env = read_env_variables()?;
    let db_pool = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;

    let (node_client, node_id) = wallet::node::register(db_pool.clone(), &node_url).await?;

    let mut wallet_ops = WalletOps::new(db_pool.clone(), node_id, node_client);

    let seed_phrase = wallet_ops.init()?;
    wallet_ops
        .mint(20.into(), starknet_types::Asset::Strk, env)
        .await?;
    let wad = wallet_ops
        .send(
            node_url,
            10.into(),
            starknet_types::Asset::Strk,
            Some("Here come the money".to_string()),
        )
        .await?;
    wallet_ops.receive(&wad).await?;
    wallet_ops
        .melt(
            5.into(),
            starknet_types::Asset::Strk,
            "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691".to_string(),
        )
        .await?;
    let pre_restore_balances = wallet_ops.balance()?;
    assert!(!pre_restore_balances.is_empty());

    let env = read_env_variables()?;
    let db_pool = db_connection()?;
    let node_url = NodeUrl::from_str(&env.node_url)?;

    let (node_client, node_id) = wallet::node::register(db_pool.clone(), &node_url).await?;
    let mut wallet_ops = WalletOps::new(db_pool.clone(), node_id, node_client);

    assert!(wallet_ops.balance()?.is_empty());
    wallet_ops.restore(seed_phrase).await?;
    let post_restore_balances = wallet_ops.balance()?;
    assert_eq!(pre_restore_balances, post_restore_balances);

    let wad = wallet_ops
        .send(
            node_url,
            10.into(),
            starknet_types::Asset::Strk,
            Some("Here come the money".to_string()),
        )
        .await?;

    // Verify the wad contains valid DLEQ proofs (Carol's scenario)
    let verification_result = wallet_ops.verify_wad_dleq_proofs(&wad).await?;
    assert!(matches!(
        verification_result,
        DleqVerificationResult::AllValid
    ));

    wallet_ops.receive(&wad).await?;

    Ok(())
}
