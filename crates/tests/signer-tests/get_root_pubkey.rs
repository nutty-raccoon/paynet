use anyhow::Result;
use signer::{GetRootPubKeyRequest, GetRootPubKeyResponse};
use signer_tests::init_signer_client;

#[tokio::test]
async fn get_root_pubkey() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("signer.env")
            .inspect_err(|e| println!("dotenvy initialization failed: {e}"));
    }

    let mut client = init_signer_client().await?;
    let res = client.get_root_pub_key(GetRootPubKeyRequest {}).await?;
    let get_root_pubkey_response: GetRootPubKeyResponse = res.into_inner();
    assert_eq!(
        get_root_pubkey_response.root_pubkey,
        "03915919cf8c316d50424df508c2b64d8e3d1ea7d55bbb6832df9b931cdfbcedd5"
    );

    Ok(())
}

#[tokio::test]
async fn get_root_pubkey_non_equal() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("signer.env")
            .inspect_err(|e| println!("dotenvy initialization failed: {e}"));
    }

    let mut client = init_signer_client().await?;
    let res = client.get_root_pub_key(GetRootPubKeyRequest {}).await?;
    let get_root_pubkey_response: GetRootPubKeyResponse = res.into_inner();
    assert_ne!(
        get_root_pubkey_response.root_pubkey,
        "03915919cf8c316d50424df508c2b64d8e3d1ea7d55bbb6832df9b931cdfbcedd4"
    );

    Ok(())
}
