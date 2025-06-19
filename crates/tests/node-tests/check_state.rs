use sqlx::postgres::PgPoolOptions;

use anyhow::Result;
use node::{
    BlindedMessage, CheckStateRequest, GetKeysRequest, GetKeysetsRequest, MintQuoteRequest,
    MintRequest, Proof, SwapRequest,
};

use node_tests::init_node_client;
use nuts::Amount;
use nuts::dhke::{blind_message, hash_to_curve, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
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

    // check token state, now is unspent
    let ys = vec![hash_to_curve(secret.as_bytes())?.to_bytes().to_vec()];

    let state = client
        .check_state(CheckStateRequest { ys: ys.clone() })
        .await?
        .into_inner();

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
    let (blinded_secret, _) = blind_message(new_secret.as_bytes(), None)?;

    let swap_request = SwapRequest {
        inputs: vec![proof],
        outputs: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        }],
    };

    let _ = client.swap(swap_request.clone()).await?.into_inner();

    // check token state, now is spent after swap
    let state = client
        .check_state(CheckStateRequest { ys })
        .await?
        .into_inner();
    assert_eq!(
        ProofState::Spent,
        state.states.first().unwrap().state.into()
    );

    Ok(())
}
