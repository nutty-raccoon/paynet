use anyhow::{Result, anyhow};
use node_tests::init_health_client;
use tonic_health::pb::{HealthCheckRequest, health_check_response::ServingStatus};

#[tokio::test]
async fn ok() -> Result<()> {
    let mut client = init_health_client().await?;
    let res = client
        .check(HealthCheckRequest {
            service: "node.Node".to_string(),
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
