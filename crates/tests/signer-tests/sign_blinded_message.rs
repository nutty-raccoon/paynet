use anyhow::{Ok, Result};
use nuts::{
    Amount,
    nut00::BlindedMessage,
    nut01::PublicKey,
    nut02::{Error, KeySetVersion, KeysetId},
};
use signer::SignBlindedMessagesRequest;
use signer::{DeclareKeysetRequest, DeclareKeysetResponse};
use signer_tests::init_signer_client;
use std::str::FromStr;

#[tokio::test]
async fn ok() -> Result<()> {
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
    let public_keys: Vec<String> = declare_keyset_response
        .keys
        .iter()
        .map(|k| k.pubkey.clone())
        .collect();

    let public_key = PublicKey::from_str(&public_keys[0]).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret: public_key,
    };

    let blind_signatures = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert!(blind_signatures.is_ok());

    Ok(())
}

#[tokio::test]
async fn invalid_secret() -> Result<()> {
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
    let public_keys: Vec<String> = declare_keyset_response
        .keys
        .iter()
        .map(|k| k.pubkey.clone())
        .collect();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret: PublicKey::from_str(&public_keys[0]).unwrap(),
    };

    let blind_signatures = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: (&[
                        3, 85, 24, 29, 104, 49, 180, 12, 119, 81, 28, 46, 22, 174, 183, 57, 39, 29,
                        73, 2, 238, 102, 75, 12, 73, 112, 82, 223, 115, 145, 247, 121,
                    ])
                        .to_vec(),
                })
                .collect(),
        })
        .await;

    assert!(blind_signatures.is_err());
    assert!(
        matches!(blind_signatures, Err(ref e) if e.code() == tonic::Code::InvalidArgument && e.message() == "Invalid secret")
    );

    Ok(())
}

#[tokio::test]
async fn amount_not_present() -> Result<()> {
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
    let public_keys: Vec<String> = declare_keyset_response
        .keys
        .iter()
        .map(|k| k.pubkey.clone())
        .collect();

    let public_key = PublicKey::from_str(&public_keys[0]).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::from_i64_repr(7),
        keyset_id,
        blinded_secret: public_key,
    };

    let blind_signatures = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert!(blind_signatures.is_err());
    assert!(
        matches!(blind_signatures, Err(ref e) if e.code() == tonic::Code::NotFound && e.message() == format!("Amount 7 not found in keyset with id {}",keyset_id))
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

    let keyset_id = KeysetId::new(
        KeySetVersion::try_from(0).unwrap(),
        [87, 81, 63, 81, 28, 247, 196],
    );

    let public_keys: Vec<String> = declare_keyset_response
        .keys
        .iter()
        .map(|k| k.pubkey.clone())
        .collect();

    let public_key = PublicKey::from_str(&public_keys[0]).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret: public_key,
    };

    let blind_signatures = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: bm.keyset_id.to_bytes().to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert!(blind_signatures.is_err());
    assert!(matches!(blind_signatures, Err(ref e) if e.code() == tonic::Code::NotFound));
    assert!(
        matches!(blind_signatures, Err(ref e) if e.code() == tonic::Code::NotFound && e.message() == format!("Keyset with id {} not found",keyset_id))
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
    let public_keys: Vec<String> = declare_keyset_response
        .keys
        .iter()
        .map(|k| k.pubkey.clone())
        .collect();

    let public_key = PublicKey::from_str(&public_keys[0]).unwrap();

    let blinded_message = BlindedMessage {
        amount: Amount::ONE,
        keyset_id,
        blinded_secret: public_key,
    };

    let blind_signatures = client
        .sign_blinded_messages(SignBlindedMessagesRequest {
            messages: [blinded_message]
                .iter()
                .map(|bm| signer::BlindedMessage {
                    amount: bm.amount.into(),
                    keyset_id: (&[1, 87, 81, 63, 81, 28, 247, 186]).to_vec(),
                    blinded_secret: bm.blinded_secret.to_bytes().to_vec(),
                })
                .collect(),
        })
        .await;

    assert!(blind_signatures.is_err());
    assert!(
        matches!(blind_signatures, Err(ref e) if e.code() == tonic::Code::InvalidArgument && e.message() == "Invalid keyset id")
    );

    Ok(())
}
