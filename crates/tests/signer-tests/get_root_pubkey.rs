use anyhow::Result;
use signer::{GetRootPubKeyRequest, GetRootPubKeyResponse};
use signer_tests::init_signer_client;

#[tokio::test]
async fn get_root_pubkey() -> Result<()> {
    let mut client = init_signer_client().await?;
    let res = client.get_root_pub_key(GetRootPubKeyRequest {}).await?;
    let get_root_pubkey_response: GetRootPubKeyResponse = res.into_inner();
    assert_eq!(
        get_root_pubkey_response.root_pubkey,
        "0x0000000000000000000000000000000000000000000000000000000000000000"
    );

    Ok(())
}
