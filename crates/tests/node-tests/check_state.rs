use sqlx::{PgPool, postgres::PgPoolOptions};
use std::str::FromStr;

use anyhow::Result;
use node::{
    AcknowledgeRequest, BlindedMessage, CheckStateRequest, GetKeysRequest, GetKeysetsRequest,
    MeltRequest, MintQuoteRequest, MintRequest, Proof, SwapRequest, hash_melt_request,
    hash_mint_request, hash_swap_request,
};
use nuts::nut02::KeysetId;

use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::{PublicKey, SecretKey};
use nuts::nut07::ProofState;
use starknet_types::Unit;

#[tokio::test]
async fn works() -> Result<()> {
    let mut client = init_node_client().await?;
    let amount = Amount::from_i64_repr(32);

    // MINT QUOTE
    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };
    let mint_quote_response = client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

    // MINT
    let keysets = client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    let secret = Secret::generate();
    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: mint_quote_response.quote,
        outputs: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        }],
    };

    let original_mint_response = client.mint(mint_request.clone()).await?.into_inner();
    let alice_keyset_id = nuts::nut02::KeysetId::from_bytes(&active_keyset.id)?;
    let _ = dotenvy::from_filename("node.env")
        .inspect_err(|e| eprintln!("Failed to load .env file: {}", e));
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(std::env::var("PG_URL").unwrap().as_str())
        .await?;

    let mut conn = pg_pool.acquire().await?;

    db_node::proof::insert_proof(
        &mut conn,
        hash_to_curve(secret.as_bytes())?,
        alice_keyset_id,
        amount.into_i64_repr(),
        secret.clone(),
        unblind_message(
            &PublicKey::from_slice(
                &original_mint_response
                    .signatures
                    .first()
                    .unwrap()
                    .blind_signature,
            )?,
            &r,
            &PublicKey::from_hex(
                &client
                    .keys(GetKeysRequest {
                        keyset_id: Some(active_keyset.id.clone()),
                    })
                    .await?
                    .into_inner()
                    .keysets
                    .first()
                    .unwrap()
                    .keys
                    .iter()
                    .find(|key| Amount::from(key.amount) == amount)
                    .unwrap()
                    .pubkey,
            )?,
        )?,
        ProofState::Unspent,
    )
    .await?;

    // check token state, now is unspent
    let ys = hash_to_curve(secret.as_bytes()).expect("hash to curve failed");
    let state = client
        .check_state(CheckStateRequest {
            ys: vec![ys.to_bytes().to_vec()],
        })
        .await?
        .into_inner();

    println!("State: {:?}", state);
    assert_eq!(
        ProofState::Unspent,
        state.states.first().unwrap().state.into()
    );

    // SWAP
    let node_pubkey_for_amount = PublicKey::from_hex(
        &client
            .keys(GetKeysRequest {
                keyset_id: Some(active_keyset.id.clone()),
            })
            .await?
            .into_inner()
            .keysets
            .first()
            .unwrap()
            .keys
            .iter()
            .find(|key| Amount::from(key.amount) == amount)
            .unwrap()
            .pubkey,
    )?;
    let blind_signature = PublicKey::from_slice(
        &original_mint_response
            .signatures
            .first()
            .unwrap()
            .blind_signature,
    )
    .unwrap();
    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;
    let proof = Proof {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        secret: secret.to_string(),
        unblind_signature: unblinded_signature.to_bytes().to_vec(),
    };

    let new_secret = Secret::generate();
    let (blinded_secret, r) = blind_message(new_secret.as_bytes(), None)?;
    let blind_message = BlindedMessage {
        amount: amount.into(),
        keyset_id: active_keyset.id.clone(),
        blinded_secret: blinded_secret.to_bytes().to_vec(),
    };

    let swap_request = SwapRequest {
        inputs: vec![proof],
        outputs: vec![blind_message],
    };
    let _ = client.swap(swap_request.clone()).await?.into_inner();
    // check token state, now is spent
    let ys = hash_to_curve(secret.as_bytes()).expect("hash to curve failed");
    let state = client
        .check_state(CheckStateRequest {
            ys: vec![ys.to_bytes().to_vec()],
        })
        .await?
        .into_inner();

    assert_eq!(
        ProofState::Spent,
        state.states.first().unwrap().state.into()
    );
    Ok(())
}
