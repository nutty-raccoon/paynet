use anyhow::Result;
use bitcoin::bip32::Xpriv;
use bitcoin::key::Secp256k1;
use nuts::nut01::PublicKey;
use signer::{GetRootPubKeyRequest, GetRootPubKeyResponse};
use signer_tests::init_signer_client;
use std::env;
use std::str::FromStr;

#[tokio::test]
async fn get_root_pubkey() -> Result<()> {
    let mut client = init_signer_client().await?;
    let res = client.get_root_pub_key(GetRootPubKeyRequest {}).await?;
    let get_root_pubkey_response: GetRootPubKeyResponse = res.into_inner();

    let root_key = env::var("ROOT_KEY").expect("ROOT_KEY must be set");
    let xpriv = Xpriv::from_str(&root_key).expect("Invalid private key");

    let secp256k1 = Secp256k1::new();
    let pubkey = xpriv.private_key.public_key(&secp256k1);

    let pubkey_hex = pubkey.to_string();
    let pubkey = PublicKey::from_str(&pubkey_hex).expect("Invalid public key hex");

    assert_eq!(get_root_pubkey_response.root_pubkey, pubkey.to_string());

    Ok(())
}
