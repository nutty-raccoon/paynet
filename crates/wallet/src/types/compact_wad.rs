use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use num_traits::CheckedAdd;

use nuts::Amount;
use nuts::nut00::secret::Secret;
use nuts::nut00::{Proof, Proofs};
use nuts::nut01::PublicKey;
use nuts::nut02::KeysetId;
use nuts::traits::Unit;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use bitcoin::base64::engine::{GeneralPurpose, general_purpose};
use bitcoin::base64::{Engine as _, alphabet};

use rusqlite;

use super::NodeUrl;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the total amount of this wad is to big")]
    WadValueOverflow,
    #[error("unsuported wad format. Should start with {PAYNET_PREFIX}")]
    UnsupportedWadFormat,
    #[error("failed to decode the base64 wad representation: {0}")]
    InvalidBase64(#[from] bitcoin::base64::DecodeError),
    #[error("failed to deserialize the CBOR wad representation: {0}")]
    InvalidCbor(#[from] ciborium::de::Error<std::io::Error>),
}

/// Token V4
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactWad<U: Unit> {
    /// Mint Url
    #[serde(rename = "n")]
    pub node_url: NodeUrl,
    /// Token Unit
    #[serde(rename = "u")]
    pub unit: U,
    /// Memo for token
    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Proofs grouped by keyset_id
    #[serde(rename = "p")]
    pub proofs: Vec<CompactKeysetProofs>,
}

impl<U: Unit> CompactWad<U> {
    /// Proofs from token
    pub fn proofs(&self) -> Proofs {
        self.proofs
            .iter()
            .flat_map(|token| token.proofs.iter().map(|p| p.proof(&token.keyset_id)))
            .collect()
    }

    /// Value
    #[inline]
    pub fn value(&self) -> Result<Amount, Error> {
        let mut sum = Amount::ZERO;
        for token in self.proofs.iter() {
            for proof in token.proofs.iter() {
                sum = sum
                    .checked_add(&proof.amount)
                    .ok_or(Error::WadValueOverflow)?;
            }
        }

        Ok(sum)
    }

    /// Memo
    #[inline]
    pub fn memo(&self) -> &Option<String> {
        &self.memo
    }

    /// Unit
    #[inline]
    pub fn unit(&self) -> &U {
        &self.unit
    }
}

pub const PAYNET_PREFIX: &str = "paynetB";

impl<U: Unit + Serialize> fmt::Display for CompactWad<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use serde::ser::Error;
        let mut data = Vec::new();
        ciborium::into_writer(self, &mut data).map_err(|e| fmt::Error::custom(e.to_string()))?;
        let encoded = general_purpose::URL_SAFE.encode(data);
        write!(f, "{}{}", PAYNET_PREFIX, encoded)
    }
}

impl<U: Unit + DeserializeOwned> FromStr for CompactWad<U> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix(PAYNET_PREFIX)
            .ok_or(Error::UnsupportedWadFormat)?;

        let decode_config = general_purpose::GeneralPurposeConfig::new()
            .with_decode_padding_mode(bitcoin::base64::engine::DecodePaddingMode::Indifferent);
        let decoded = GeneralPurpose::new(&alphabet::URL_SAFE, decode_config).decode(s)?;
        let token = ciborium::from_reader(&decoded[..])?;
        Ok(token)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactKeysetProofs {
    /// `Keyset id`
    #[serde(
        rename = "i",
        serialize_with = "serialize_keyset_id_as_bytes",
        deserialize_with = "deserialize_keyset_id_from_bytes"
    )]
    pub keyset_id: KeysetId,
    /// Proofs
    #[serde(rename = "p")]
    pub proofs: Vec<CompactProof>,
}

fn serialize_keyset_id_as_bytes<S>(keyset_id: &KeysetId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&keyset_id.to_bytes())
}

fn deserialize_keyset_id_from_bytes<'de, D>(deserializer: D) -> Result<KeysetId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = Vec::<u8>::deserialize(deserializer)?;
    KeysetId::from_bytes(&bytes).map_err(|_| {
        serde::de::Error::invalid_value(
            serde::de::Unexpected::Bytes(&bytes),
            &"bytes of a valid keyset id",
        )
    })
}

/// Proof V4
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactProof {
    /// Amount in satoshi
    #[serde(rename = "a")]
    pub amount: Amount,
    /// Secret message
    #[serde(rename = "s")]
    pub secret: Secret,
    /// Unblinded signature
    #[serde(
        serialize_with = "serialize_pubkey_as_bytes",
        deserialize_with = "deserialize_pubkey_from_bytes"
    )]
    pub c: PublicKey,
}

impl CompactProof {
    /// [`ProofV4`] into [`Proof`]
    pub fn proof(&self, keyset_id: &KeysetId) -> Proof {
        Proof {
            amount: self.amount,
            keyset_id: *keyset_id,
            secret: self.secret.clone(),
            c: self.c,
        }
    }
}

fn serialize_pubkey_as_bytes<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&key.to_bytes())
}

fn deserialize_pubkey_from_bytes<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = Vec::<u8>::deserialize(deserializer)?;
    PublicKey::from_slice(&bytes).map_err(serde::de::Error::custom)
}

/// Validates that all proofs have amounts less than the max_order (maximum key amount) for their keyset.
///
/// # Arguments
/// * `db_conn` - A reference to the database connection.
/// * `proofs` - A slice of proofs to validate.
///
/// # Returns
/// * `Ok(())` if all proofs are valid.
/// * `Err(String)` if any proof amount is not less than the max_order for its keyset.
pub fn validate_proofs_under_max_order_for_keyset(
    db_conn: &rusqlite::Connection,
    proofs: &[nuts::nut00::Proof],
) -> Result<(), String> {
    use nuts::nut02::KeysetId;
    let mut keyset_to_max: HashMap<KeysetId, u64> = HashMap::new();
    for proof in proofs {
        let keyset_id = proof.keyset_id;
        let max_order = *keyset_to_max.entry(keyset_id).or_insert_with(|| {
            let mut stmt = db_conn
                .prepare("SELECT MAX(amount) FROM key WHERE keyset_id = ?1")
                .expect("Failed to prepare statement for max amount");
            stmt.query_row([keyset_id], |row| row.get::<_, Option<u64>>(0))
                .expect("Failed to query max amount")
                .unwrap_or(0)
        });
        let proof_amount = u64::from(proof.amount);
        if proof_amount >= max_order {
            return Err(format!(
                "Proof amount {} is not less than max_order {} for keyset {}",
                proof_amount, max_order, keyset_id
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuts::{Amount, nut00::Proof, nut02::KeysetId};
    use rusqlite::Connection;

    pub fn test_pubkey() -> nuts::nut01::PublicKey {
        nuts::nut01::PublicKey::from_hex(
            "02194603ffa36356f4a56b7df9371fc3192472351453ec7398b8da8117e7c3e104",
        )
        .expect("valid test pubkey")
    }

    fn setup_db_with_keys(keys: &[(KeysetId, u64)]) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE key (keyset_id BLOB(8), amount INTEGER, pubkey BLOB(33), PRIMARY KEY (keyset_id, amount))",
            [],
        ).unwrap();
        for (keyset_id, amount) in keys {
            conn.execute(
                "INSERT INTO key (keyset_id, amount, pubkey) VALUES (?1, ?2, zeroblob(33))",
                (&keyset_id.to_bytes()[..], *amount as i64),
            )
            .unwrap();
        }
        conn
    }

    fn make_proof(keyset_id: KeysetId, amount: u64) -> Proof {
        let dummy_pubkey = test_pubkey();
        Proof {
            amount: Amount::from(amount),
            keyset_id,
            secret: Default::default(),
            c: dummy_pubkey,
        }
    }

    #[test]
    fn test_all_proofs_amounts_valid_for_keyset_positive() {
        let keyset_id = KeysetId::from_str("009a1f293253e41e").unwrap();
        let keys = vec![(keyset_id, 1), (keyset_id, 2), (keyset_id, 4)];
        let conn = setup_db_with_keys(&keys);
        let proofs = vec![make_proof(keyset_id, 1), make_proof(keyset_id, 2)];
        assert!(validate_proofs_under_max_order_for_keyset(&conn, &proofs).is_ok());
    }

    #[test]
    fn test_all_proofs_amounts_valid_for_keyset_negative() {
        let keyset_id = KeysetId::from_str("009a1f293253e41e").unwrap();
        let keys = vec![(keyset_id, 1), (keyset_id, 2), (keyset_id, 4)];
        let conn = setup_db_with_keys(&keys);
        let proofs = vec![make_proof(keyset_id, 1), make_proof(keyset_id, 8)];
        let res = validate_proofs_under_max_order_for_keyset(&conn, &proofs);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("8"));
    }

    #[test]
    fn test_all_proofs_amounts_valid_for_keyset_multiple_keysets() {
        let keyset1 = KeysetId::from_str("009a1f293253e41e").unwrap();
        let keyset2 = KeysetId::from_str("001e14e3a292f190").unwrap();
        let keys = vec![(keyset1, 2), (keyset2, 4)];
        let conn = setup_db_with_keys(&keys);
        // Use proof amounts strictly less than max_order for each keyset
        let proofs = vec![make_proof(keyset1, 1), make_proof(keyset2, 3)];
        assert!(validate_proofs_under_max_order_for_keyset(&conn, &proofs).is_ok());
        let proofs_invalid = vec![make_proof(keyset1, 2), make_proof(keyset2, 8)];
        assert!(validate_proofs_under_max_order_for_keyset(&conn, &proofs_invalid).is_err());
    }
}
