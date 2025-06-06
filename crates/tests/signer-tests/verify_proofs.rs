use anyhow::{Ok, Result};
use node::{
    BlindedMessage, GetKeysRequest, GetKeysetsRequest, Keyset, MintQuoteRequest, MintRequest,
};
use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use signer::{Proof, VerifyProofsRequest, VerifyProofsResponse};
use signer_tests::{init_node_client, init_signer_client};
use starknet_types::Unit;

async fn create_valid_proof(amount: Amount) -> Result<(Proof, Keyset)> {
    let mut node_client = init_node_client().await?;
    let secret = Secret::generate();
    let keysets = node_client
        .keysets(GetKeysetsRequest {})
        .await?
        .into_inner()
        .keysets;
    let active_keyset = keysets
        .iter()
        .find(|ks| ks.active && ks.unit == Unit::MilliStrk.as_str())
        .unwrap();

    let node_pubkey_for_amount = PublicKey::from_hex(
        &node_client
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

    let mint_quote_request = MintQuoteRequest {
        method: "starknet".to_string(),
        amount: amount.into(),
        unit: Unit::MilliStrk.to_string(),
        description: None,
    };

    let original_mint_quote_response = node_client
        .mint_quote(mint_quote_request.clone())
        .await?
        .into_inner();

    let (blinded_secret, r) = blind_message(secret.as_bytes(), None)?;
    let mint_request = MintRequest {
        method: "starknet".to_string(),
        quote: original_mint_quote_response.quote,
        outputs: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: active_keyset.id.clone(),
            blinded_secret: blinded_secret.to_bytes().to_vec(),
        }],
    };
    let original_mint_response = node_client.mint(mint_request.clone()).await?.into_inner();

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

    Ok((proof, active_keyset.clone()))
}

#[tokio::test]
async fn verify_ok() -> Result<()> {
    let mut signer_client = init_signer_client().await?;
    let (proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await?;
    assert!(res.get_ref().is_valid);
    Ok(())
}

#[tokio::test]
async fn verify_invalid_keyset_id_format() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;
    proof.keyset_id = b"\xF0\x5D\xB0\x25\x9D\x04\x42\xBA\xAA\xDD\x66\x7B\x80\x41\x88\xA8".to_vec();
    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn verify_empty_signature() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;
    proof.unblind_signature = vec![];
    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn verify_unknown_keyset_id() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;
    proof.keyset_id = "unknown_keyset_id".as_bytes().to_vec();

    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn verify_invalid_amount_not_power_of_two() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(8)).await?;
    proof.amount = 7;
    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await;
    assert!(res.is_err());
    Ok(())
}

#[tokio::test]
async fn verify_signature_valid_format_but_invalid_content() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;

    let last_index = proof.unblind_signature.len() - 1;
    proof.unblind_signature[last_index] ^= 0x01;

    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await?;

    assert!(!res.get_ref().is_valid);
    Ok(())
}

#[tokio::test]
async fn verify_structurally_valid_but_incorrect_signature() -> Result<()> {
    let (mut proof1, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;
    let (proof2, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;

    proof1.unblind_signature = proof2.unblind_signature.clone();

    let mut signer_client = init_signer_client().await?;
    let res = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof1],
        })
        .await?;

    assert!(!res.get_ref().is_valid);
    Ok(())
}

#[tokio::test]
async fn verify_malformed_signature() -> Result<()> {
    let (mut proof, _) = create_valid_proof(Amount::from_i64_repr(32)).await?;

    proof.unblind_signature = vec![0x99; 10];

    let mut signer_client = init_signer_client().await?;
    let result = signer_client
        .verify_proofs(VerifyProofsRequest {
            proofs: vec![proof],
        })
        .await;

    // Expect an error (invalid argument due to bad signature format)
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    assert!(
        err.message().contains("invalid signature"),
        "Unexpected error: {:?}",
        err
    );

    Ok(())
}
