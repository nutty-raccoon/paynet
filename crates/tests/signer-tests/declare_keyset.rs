use anyhow::Result;
use nuts::{nut01::PublicKey, nut02::KeysetId};
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
    assert_eq!(declare_keyset_response.keys.len(), 32);
    let mut i = 1;
    for key in declare_keyset_response.keys.iter() {
        assert_eq!(i, key.amount);
        i *= 2;
    }

    let keyset_id = KeysetId::from_iter(
        declare_keyset_response
            .keys
            .into_iter()
            .map(|k| PublicKey::from_str(&k.pubkey).unwrap()),
    );

    assert_eq!(
        keyset_id.to_bytes().to_vec(),
        declare_keyset_response.keyset_id
    );

    Ok(())
}

#[tokio::test]
async fn unknown_unit() -> Result<()> {
    let mut client = init_signer_client().await?;
    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "stark".to_string(),
            index: 1,
            max_order: 32,
        })
        .await;

    assert!(res.is_err());
    assert!(matches!(res, Err(status) if status.code() == tonic::Code::InvalidArgument ));

    Ok(())
}

#[tokio::test]
async fn exceed_max_order() -> Result<()> {
    let mut client = init_signer_client().await?;
    let res = client
        .declare_keyset(DeclareKeysetRequest {
            unit: "strk".to_string(),
            index: 1,
            max_order: 300,
        })
        .await;

    assert!(res.is_err());
    assert!(matches!(res, Err(status) if status.code() == tonic::Code::InvalidArgument ));

    Ok(())
}
