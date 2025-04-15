use anyhow::Result;
use serde_json;
use wallet::types::compact_wad::{CompactWad, PAYNET_PREFIX};
use wallet::types::Wad;
use nuts::Amount;
use super::{create_test_wad, TestUnit};
use std::io::Cursor;

#[test]
fn test_compact_wad_serialization() -> Result<()> {
    let wad = create_test_wad(true);
    {
        let serialized = serde_json::to_string(&wad)?;
        let cursor = Cursor::new(serialized.clone());
        let deserialized: CompactWad<TestUnit> = serde_json::from_reader(cursor)?;
        
        // Verify format
        assert!(serialized.contains(PAYNET_PREFIX));
        
        // Verify roundtrip
        assert_eq!(deserialized.node_url, wad.node_url);
        assert_eq!(deserialized.unit, wad.unit);
        assert_eq!(deserialized.memo, wad.memo);
        assert_eq!(deserialized.proofs.len(), wad.proofs.len());
    }
    Ok(())
}

#[test]
fn test_compact_wad_without_memo() -> Result<()> {
    let wad = create_test_wad(false);
    {
        let serialized = serde_json::to_string(&wad)?;
        let cursor = Cursor::new(serialized.clone());
        let deserialized: CompactWad<TestUnit> = serde_json::from_reader(cursor)?;
        
        assert_eq!(deserialized.memo, None);
    }
    Ok(())
}

#[test]
fn test_compact_wad_proofs_conversion() -> Result<()> {
    let wad = create_test_wad(false);
    let keyset_id = wad.proofs[0].keyset_id;
    
    let proofs = wad.proofs();
    assert_eq!(proofs.len(), 1);
    
    let proof = &proofs[0];
    assert_eq!(proof.keyset_id, keyset_id);
    assert_eq!(proof.amount, Amount::from(100u64));
    
    Ok(())
}

#[test]
fn test_decode_wad() -> Result<()> {
    let compact_wad = create_test_wad(true);
    {
        let serialized = serde_json::to_string(&compact_wad)?;
        let cursor = Cursor::new(serialized.clone());
        let wad: CompactWad<TestUnit> = serde_json::from_reader(cursor)?;
        
        // Check all fields are correctly decoded
        assert_eq!(wad.node_url, compact_wad.node_url);
        assert_eq!(wad.unit.0, "BTC");
        assert_eq!(wad.memo, Some("Test memo".to_string()));
        assert_eq!(wad.value()?, Amount::from(100u64));
        
        // Check conversion to regular wad
        let node_url = wad.node_url.clone();
        let proofs = wad.proofs();
        let regular_wad = Wad {
            node_url,
            proofs,
        };
        assert_eq!(regular_wad.proofs.len(), 1);
        assert_eq!(regular_wad.proofs[0].amount, Amount::from(100u64));
    }
    Ok(())
}
