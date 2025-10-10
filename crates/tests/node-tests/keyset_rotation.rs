use anyhow::Result;
use cashu_client::{CashuClient, ClientKeysResponse, ClientKeysetsResponse};
use node_client::RotateKeysetsRequest;
use node_tests::{init_keyset_client, init_node_client};
use nuts::nut02::KeysetId;
use std::collections::HashMap;

#[tokio::test]
async fn ok() -> Result<()> {
    let mut node_client = init_node_client().await?;
    let mut keyset_client = init_keyset_client().await?;

    // Existing keysets before rotation
    let res = node_client.keysets().await?;
    let get_keysets_response: ClientKeysetsResponse = res;

    assert!(
        !get_keysets_response.keysets.is_empty(),
        "No keysets found before rotation"
    );

    // Store old keysets for comparison
    let mut old_keysets: HashMap<Vec<u8>, bool> = HashMap::new();

    for keyset in &get_keysets_response.keysets {
        old_keysets.insert(keyset.id.clone(), keyset.active);
    }

    // trigger rotate keysets
    let _ = keyset_client
        .rotate_keysets(RotateKeysetsRequest {})
        .await?;

    // Check that old keysets are deactivated
    for (old_id, was_active) in &old_keysets {
        if *(was_active) {
            let get_keys_response: ClientKeysResponse = node_client
                .keys(Some(KeysetId::from_bytes(&old_id.clone()).unwrap()))
                .await?;

            let keyset = get_keys_response
                .keysets
                .first()
                .expect("Expected at least one keyset");

            assert!(
                !keyset.active,
                "Old keyset with ID {:?} is still active",
                old_id
            );
        }
    }

    // get all keysets
    let res = node_client.keysets().await?;
    let curr_keysets_response: ClientKeysetsResponse = res;

    for keyset in &curr_keysets_response.keysets {
        if !old_keysets.contains_key(&keyset.id) {
            assert!(keyset.active, "New keyset {:?} is not active!", keyset.id);
        } else {
            assert!(!keyset.active, "Old keyset {:?} is active!", keyset.id);
        }
    }

    Ok(())
}
