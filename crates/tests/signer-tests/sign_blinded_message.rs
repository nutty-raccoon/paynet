use anyhow::{Ok, Result};
use nuts::{
    Amount,
    nut00::BlindedMessage,
    nut01::PublicKey,
    nut02::{KeySetVersion, KeysetId},
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
    // println!("{:?}",declare_keyset_response);

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
    println!("{:?}", public_keys);

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
        .await?
        .into_inner()
        .signatures;

    println!("{:?}", blind_signatures);
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

    let public_key =
        PublicKey::from_str("031d032d4042b010310f9ed54b12c819afc334505752e9e31a9fb13a43ad2eea2");

    assert!(public_key.is_err());

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

    let keyset_id =
        KeysetId::from_str("034a013c42af6d4ae91e67de936d6039e0e35899d0c3936a59c9292b9646adf68");

    assert!(keyset_id.is_err());
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

    let keyset_version = KeySetVersion::try_from(1);

    assert!(keyset_version.is_err());

    Ok(())
}
