use anyhow::Result;
use node::{
    GetKeysRequest, GetKeysResponse, GetKeysetsRequest, GetKeysetsResponse, RotateKeysetsRequest,
};
use node_tests::{init_keyset_client, init_node_client};
use std::collections::HashMap;

#[tokio::test]
async fn ok() -> Result<()> {
    let mut node_client = init_node_client().await?;
    let mut client = init_keyset_client().await?;

    // Existing keysets before rotation
    let res = node_client.keysets(GetKeysetsRequest {}).await?;
    let get_keysets_response: GetKeysetsResponse = res.into_inner();

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
    let _ = client.rotate_keysets(RotateKeysetsRequest {}).await?;

    // get new keysets
    let res = node_client.keysets(GetKeysetsRequest {}).await?;

    let curr_keysets_response: GetKeysetsResponse = res.into_inner();

    // Check that old keysets are deactivated
    for (old_id, was_active) in &old_keysets {
        if *(was_active) {
            let res = node_client
                .keys(GetKeysRequest {
                    keyset_id: Some(old_id.clone()),
                })
                .await?;

            let old_keyset: GetKeysResponse = res.into_inner();

            assert!(
                old_keyset.keysets.iter().all(|k| !k.active),
                "Old keyset with ID {:?} is still active",
                old_id
            )
        }
    }

    // check that new keysets are active
    let mut new_keys_found = false;
    for keyset in &curr_keysets_response.keysets {
        if !old_keysets.contains_key(&keyset.id) {
            assert!(keyset.active, "New keyset {:?} is not active!", keyset.id);
            new_keys_found = true;
        }
    }

    assert!(new_keys_found, "No new active keysets were created!");

    Ok(())
}
