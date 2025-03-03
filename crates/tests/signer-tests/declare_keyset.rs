use anyhow::{Result, anyhow};
use nuts::{nut01::PublicKey, nut02::KeysetId};
use signer::{DeclareKeysetRequest, DeclareKeysetResponse};
use signer_tests::{init_health_client, init_signer_client};
use std::str::FromStr;
use tonic_health::pb::{HealthCheckRequest, health_check_response::ServingStatus};

#[tokio::test]
async fn is_healthy_works() -> Result<()> {
    let mut client = init_health_client().await?;
    let res = client
        .check(HealthCheckRequest {
            service: "signer.Signer".to_string(),
        })
        .await?;
    let serving_status = ServingStatus::try_from(res.into_inner().status)?;

    if serving_status == ServingStatus::Serving {
        Ok(())
    } else {
        Err(anyhow!(
            "invalid status, expected SERVING, got {}",
            serving_status.as_str_name()
        ))
    }
}

#[tokio::test]
async fn declare_keyset_works() -> Result<()> {
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
