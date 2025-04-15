use std::{fmt::{self, Display}, str::FromStr};
use nuts::{Amount, nut00::secret::Secret, nut01::PublicKey, nut02::KeysetId, traits::Unit};
use serde::{Serialize, Deserialize};
use wallet::types::{
    compact_wad::{CompactWad, CompactKeysetProofs, CompactProof},
};

#[cfg(test)]
mod tests {
    use super::*;
    include!("../compact_wad.rs");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestUnit(String);

impl Display for TestUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TestUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BTC" => Ok(TestUnit("BTC".to_string())),
            _ => Ok(TestUnit("UNKNOWN".to_string())),
        }
    }
}

impl From<TestUnit> for u32 {
    fn from(unit: TestUnit) -> Self {
        match unit.0.as_str() {
            "BTC" => 1,
            _ => 0,
        }
    }
}

impl Unit for TestUnit {
    // Unit trait is a marker trait, no methods needed
}

impl From<&str> for TestUnit {
    fn from(s: &str) -> Self {
        match s {
            "BTC" => TestUnit("BTC".to_string()),
            _ => TestUnit("UNKNOWN".to_string()),
        }
    }
}

pub fn create_test_wad(with_memo: bool) -> CompactWad<TestUnit> {
    CompactWad {
        node_url: "http://localhost:8080".parse().unwrap(),
        unit: TestUnit::from("BTC"),
        memo: if with_memo { Some("Test memo".to_string()) } else { None },
        proofs: vec![
            CompactKeysetProofs {
                keyset_id: KeysetId::from_bytes(&[1; 32]).unwrap(),
                proofs: vec![
                    CompactProof {
                        amount: Amount::from(100u64),
                        secret: Secret::new("test_secret").unwrap(),
                        c: PublicKey::from_slice(&[3; 32]).unwrap(),
                    }
                ],
            }
        ],
    }
}
