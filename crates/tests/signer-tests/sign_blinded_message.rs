use anyhow::{Ok, Result};
use assert_matches::assert_matches;
use nuts::{
    Amount,
    dhke::blind_message,
    nut00::{BlindedMessage, secret::Secret},
    nut01::PublicKey,
    nut02::KeysetId,
};
use signer::SignBlindedMessagesRequest;
use signer::{DeclareKeysetRequest, DeclareKeysetResponse};
use signer_tests::init_signer_client;
use std::str::FromStr;
use tonic::Code;

#[tokio::test]
async fn secret() -> Result<()> {
    let mut client = init_signer_client().await?;

    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "strk".to_string(),
            index: 1,
            max_order: 32,
        })
        .await?;

    let declare_keyset_response: DeclareKeysetResponse = res.into_inner();

    let keyset_id = KeysetId::from_iter(
        declare_keyset_response
            .clone()
            .keys
            .into_iter()
            .map(|k| PublicKey::from_str(&k.pubkey).unwrap()),
    );

    let secret = Secret::generate();
    let (blinded_secret, _secret) = blind_message(&secret.to_bytes(), None).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret,
    };

    // bad secret
    let res = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message.clone()]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: Vec::new(),
                })
                .collect(),
        })
        .await;

    assert_matches!(
        res,
        Err(s) if s.code() == Code::InvalidArgument && s.message() == "invalid secret"
    );

    // empty secret
    let res = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: "lorem ipsum".as_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert_matches!(
        res,
        Err(s) if s.code() == Code::InvalidArgument && s.message() == "invalid secret"
    );

    Ok(())
}

#[tokio::test]
async fn amount() -> Result<()> {
    let mut client = init_signer_client().await?;

    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "strk".to_string(),
            index: 1,
            max_order: 32,
        })
        .await?;

    let declare_keyset_response: DeclareKeysetResponse = res.into_inner();

    let keyset_id = KeysetId::from_iter(
        declare_keyset_response
            .clone()
            .keys
            .into_iter()
            .map(|k| PublicKey::from_str(&k.pubkey).unwrap()),
    );

    let secret = Secret::generate();
    let (blinded_secret, _secret) = blind_message(&secret.to_bytes(), None).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret,
    };

    let res = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message.clone()]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: 13,
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert_matches!(
        res,
        Err(s) if s.code() == Code::InvalidArgument && s.message() == "amount is not a power of two"
    );

    Ok(())
}

#[tokio::test]
async fn non_existent_keysetid() -> Result<()> {
    let mut client = init_signer_client().await?;

    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "strk".to_string(),
            index: 1,
            max_order: 32,
        })
        .await?;
    let declare_keyset_response: DeclareKeysetResponse = res.into_inner();

    let keyset_id = KeysetId::from_iter(
        declare_keyset_response
            .clone()
            .keys
            .into_iter()
            .map(|k| PublicKey::from_str(&k.pubkey).unwrap()),
    );

    let secret = Secret::generate();
    let (blinded_secret, _secret) = blind_message(&secret.to_bytes(), None).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret,
    };

    let res = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message.clone()]
                .iter()
                .map(|bm| {
                    let mut keyset_id = bm.keyset_id.to_bytes().to_vec();
                    keyset_id[2] = 0x0;

                    signer::BlindedMessage {
                        amount: bm.amount.into(),
                        keyset_id,
                        blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                    }
                })
                .collect(),
        })
        .await;

    assert_matches!(
        res,
        Err(s) if s.code() == Code::NotFound && s.message() == "keyset not found"
    );

    Ok(())
}

#[tokio::test]
async fn bad_version_keysetid() -> Result<()> {
    let mut client = init_signer_client().await?;

    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "strk".to_string(),
            index: 1,
            max_order: 32,
        })
        .await?;
    let declare_keyset_response: DeclareKeysetResponse = res.into_inner();

    let keyset_id = KeysetId::from_iter(
        declare_keyset_response
            .clone()
            .keys
            .into_iter()
            .map(|k| PublicKey::from_str(&k.pubkey).unwrap()),
    );

    let secret = Secret::generate();
    let (blinded_secret, _secret) = blind_message(&secret.to_bytes(), None).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret,
    };

    let res = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message.clone()]
                .iter()
                .map(|bm| {
                    let mut keyset_id = vec![0xffu8];
                    keyset_id.extend_from_slice(&bm.keyset_id.id());

                    signer::BlindedMessage {
                        amount: bm.amount.into(),
                        keyset_id,
                        blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                    }
                })
                .collect(),
        })
        .await;

    assert_matches!(
        res,
        Err(s) if s.code() == Code::InvalidArgument && s.message() == "invalid keyset id"
    );

    Ok(())
}
