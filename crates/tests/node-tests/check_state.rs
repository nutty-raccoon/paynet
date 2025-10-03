use std::vec;

use anyhow::Result;

use cashu_client::{CashuClient, ClientMintQuoteRequest};
use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut02::KeysetId;
use nuts::nut07::ProofState;
use starknet_types::Unit;

#[tokio::test]
async fn test_multiple_tokens() -> Result<()> {
    let mut client = init_node_client().await?;

    // Define multiple token amounts
    let amounts = vec![
        Amount::from_i64_repr(8),
        Amount::from_i64_repr(16),
        Amount::from_i64_repr(32),
        Amount::from_i64_repr(64),
    ];
    let total_amount = Amount::from_i64_repr(120);

    // MINT QUOTE for total amount
    let mint_quote_request = ClientMintQuoteRequest {
        method: "starknet".to_string(),
        amount: total_amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let mint_quote_response = client.mint_quote(mint_quote_request.clone()).await?;

    // Get active keyset
    let keysets = client.keysets().await?.keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    // Generate secrets and blind messages for each amount
    let mut secrets = Vec::new();
    let mut rs = Vec::new();
    let mut outputs = Vec::new();
    let mut ys = Vec::new(); // For state checking

    for amount in &amounts {
        let secret = Secret::generate();
        let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;

        // Store for later use
        secrets.push(secret.clone());
        rs.push(r);
        ys.push(hash_to_curve(secret.as_bytes())?.to_bytes().to_vec());

        outputs.push(nuts::nut00::BlindedMessage {
            amount: *amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        });
    }

    // MINT multiple tokens
    let mint_request = nuts::nut04::MintRequest {
        quote: mint_quote_response.quote,
        outputs,
    };

    let mint_response = client
        .mint(mint_request.clone(), "starknet".to_string())
        .await?;

    // Check all tokens are unspent and in correct order as input ys
    let state = client
        .check_state(cashu_client::CheckStateRequest { ys: ys.clone() })
        .await?;

    for (i, state_info) in state.proof_check_states.iter().enumerate() {
        assert_eq!(ProofState::Unspent, state_info.state);
        assert_eq!(ys[i], state_info.y.to_bytes().to_vec());
    }

    // Get node public keys for all amounts
    let get_keys_resonse = client.keys(Some(active_keyset.id.clone())).await?;
    let node_keys = &get_keys_resonse.keysets.first().unwrap().keys;

    // Create proofs for all tokens
    let mut all_proofs = Vec::new();
    for (i, amount) in amounts.iter().enumerate() {
        let node_pubkey_for_amount = node_keys
            .iter()
            .find(|key| key.amount == *amount)
            .unwrap()
            .publickey;

        let blind_signature = mint_response.signatures[i].c;

        let unblinded_signature =
            unblind_message(&blind_signature, &rs[i], &node_pubkey_for_amount)?;

        all_proofs.push(nuts::nut00::Proof {
            amount: *amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            secret: secrets[i].clone(),
            c: unblinded_signature,
        });
    }

    let proofs_to_swap = vec![all_proofs[0].clone(), all_proofs[2].clone()];
    let amounts_to_swap = vec![&amounts[0], &amounts[2]];

    let mut new_outputs = Vec::new();
    for amount in amounts_to_swap.clone() {
        let new_secret = Secret::generate();
        let (blinded_secret, _) = blind_message(new_secret.as_bytes(), None)?;

        new_outputs.push(nuts::nut00::BlindedMessage {
            amount: *amount,
            keyset_id: KeysetId::from_bytes(&active_keyset.id.clone())?,
            blinded_secret,
        });
    }

    // SWAP only selected tokens
    let swap_request = nuts::nut03::SwapRequest {
        inputs: proofs_to_swap,
        outputs: new_outputs,
    };

    let _ = client.swap(swap_request.clone()).await?;

    // Check final state: some spent, some unspent
    let final_state = client
        .check_state(cashu_client::CheckStateRequest { ys: ys.clone() })
        .await?;

    for (i, state_info) in final_state.proof_check_states.iter().enumerate() {
        if amounts_to_swap.contains(&&amounts[i]) {
            assert_eq!(ProofState::Spent, state_info.state);
        } else {
            assert_eq!(ProofState::Unspent, state_info.state);
        }
        assert_eq!(ys[i], state_info.y.to_bytes().to_vec());
    }

    Ok(())
}
