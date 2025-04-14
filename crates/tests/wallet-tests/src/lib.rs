use anyhow::Result;
use nuts::{Amount, nut00::secret::Secret, nut01::PublicKey, nut02::KeysetId};
use wallet::types::{CompactWad, CompactKeysetProofs, CompactProof, NodeUrl};

pub fn create_test_wad(with_memo: bool) -> CompactWad<String> {
    CompactWad {
        node_url: "http://localhost:8080".parse().unwrap(),
        unit: "BTC".to_string(),
        memo: if with_memo { Some("Test memo".to_string()) } else { None },
        proofs: vec![
            CompactKeysetProofs {
                keyset_id: KeysetId::from_bytes(&[1; 32]).unwrap(),
                proofs: vec![
                    CompactProof {
                        amount: Amount::from(100),
                        secret: Secret::from_bytes(&[2; 32]).unwrap(),
                        c: PublicKey::from_slice(&[3; 32]).unwrap(),
                    }
                ],
            }
        ],
    }
}
