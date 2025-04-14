use anyhow::Result;
use wallet_tests::create_test_wad;

#[test]
fn test_compact_wad_serialization() -> Result<()> {
    let wad = create_test_wad(true);
    let serialized = wad.to_string();
    
    // Verify format
    assert!(serialized.starts_with("paynetB"));
    
    // Verify roundtrip
    let deserialized: CompactWad<String> = serialized.parse()?;
    assert_eq!(deserialized.node_url, wad.node_url);
    assert_eq!(deserialized.unit, wad.unit);
    assert_eq!(deserialized.memo, wad.memo);
    assert_eq!(deserialized.proofs.len(), wad.proofs.len());
    
    Ok(())
}

#[test]
fn test_compact_wad_without_memo() -> Result<()> {
    let wad = create_test_wad(false);
    let serialized = wad.to_string();
    
    let deserialized: CompactWad<String> = serialized.parse()?;
    assert_eq!(deserialized.memo, None);
    
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
    assert_eq!(proof.amount, Amount::from(100));
    
    Ok(())
}
