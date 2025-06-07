use anyhow::{Ok, Result};

use nuts::Amount;
use nuts::dhke::{blind_message, unblind_message};
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use signer::{
    BlindedMessage, DeclareKeysetRequest, DeclareKeysetResponse, Proof, SignBlindedMessagesRequest,
    SignBlindedMessagesResponse, VerifyProofsRequest,
};
use signer_tests::init_signer_client;
use starknet_types::Unit;

async fn create_valid_proof(amount: Amount) -> Result<Proof> {
    let mut signer_client = init_signer_client().await?;

    let res = signer_client
        .declare_keyset(DeclareKeysetRequest {
            unit: Unit::MilliStrk.to_string(),
            index: 1,
            max_order: 32,
        })
        .await?;

    let declare_keyset_response: DeclareKeysetResponse = res.into_inner();

    let public_key = declare_keyset_response
        .keys
        .iter()
        .find(|key| Amount::from(key.amount) == amount)
        .ok_or_else(|| anyhow::anyhow!("No key found for amount {}", amount))?;

    let node_pubkey_for_amount = PublicKey::from_hex(&public_key.pubkey)?;

    let secret = Secret::generate();
    let (blinded_message, r) = blind_message(secret.as_bytes(), None)?;

    let sign_request = SignBlindedMessagesRequest {
        messages: vec![BlindedMessage {
            amount: amount.into(),
            keyset_id: declare_keyset_response.keyset_id.clone(),
            blinded_secret: blinded_message.to_bytes().to_vec(),
        }],
    };

    let sign_response: SignBlindedMessagesResponse = signer_client
        .sign_blinded_messages(sign_request)
        .await?
        .into_inner();

    let blind_signature = PublicKey::from_slice(
        sign_response
            .signatures
            .first()
            .ok_or_else(|| anyhow::anyhow!("No signature returned"))?,
    )?;

    let unblinded_signature = unblind_message(&blind_signature, &r, &node_pubkey_for_amount)?;

    let proof = Proof {
        amount: amount.into(),
        keyset_id: declare_keyset_response.keyset_id,
        secret: secret.to_string(),
        unblind_signature: unblinded_signature.to_bytes().to_vec(),
    };

    Ok(proof)
}

#[tokio::test]
async fn verify_ok() -> Result<()> {
    let mut signer_client = init_signer_client().await?;
    let proof = create_valid_proof(Amount::from_i64_repr(32)).await?;
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
    let mut proof = create_valid_proof(Amount::from_i64_repr(32)).await?;
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
    let mut proof = create_valid_proof(Amount::from_i64_repr(32)).await?;
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
    let mut proof = create_valid_proof(Amount::from_i64_repr(32)).await?;
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
    let mut proof = create_valid_proof(Amount::from_i64_repr(8)).await?;
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
    let mut proof = create_valid_proof(Amount::from_i64_repr(32)).await?;

    use nuts::nut01::SecretKey;

    // Generate a random valid public key (wrong signature)
    let random_secret = SecretKey::generate();
    let wrong_signature = random_secret.public_key();
    proof.unblind_signature = wrong_signature.to_bytes().to_vec();

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
    let mut proof1 = create_valid_proof(Amount::from_i64_repr(32)).await?;
    let proof2 = create_valid_proof(Amount::from_i64_repr(32)).await?;

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
    let mut proof = create_valid_proof(Amount::from_i64_repr(32)).await?;

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
