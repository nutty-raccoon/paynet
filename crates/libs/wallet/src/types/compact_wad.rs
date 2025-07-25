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

use super::NodeUrl;

use bitcoin::base64::engine::{GeneralPurpose, general_purpose};
use bitcoin::base64::{Engine as _, alphabet};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the total amount of this wad is to big")]
    WadValueOverflow,
    #[error("unsuported wad format. Should start with {CASHU_PREFIX}")]
    UnsupportedWadFormat,
    #[error("failed to decode the base64 wad representation: {0}")]
    InvalidBase64(#[from] bitcoin::base64::DecodeError),
    #[error("failed to deserialize the CBOR wad representation: {0}")]
    InvalidCbor(#[from] ciborium::de::Error<std::io::Error>),
    #[error("failed to parse individual wad token: {0}")]
    InvalidWadToken(Box<Error>),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompactWads<U: Unit>(pub Vec<CompactWad<U>>);

impl<U: Unit> CompactWads<U> {
    pub fn new(wads: Vec<CompactWad<U>>) -> Self {
        Self(wads)
    }
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

pub const CASHU_PREFIX: &str = "cashuB";

impl<U: Unit + Serialize> fmt::Display for CompactWad<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use serde::ser::Error;
        let mut data = Vec::new();
        ciborium::into_writer(self, &mut data).map_err(|e| fmt::Error::custom(e.to_string()))?;
        let encoded = general_purpose::URL_SAFE.encode(data);
        write!(f, "{}{}", CASHU_PREFIX, encoded)
    }
}

impl<U: Unit + DeserializeOwned> FromStr for CompactWad<U> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s
            .strip_prefix(CASHU_PREFIX)
            .ok_or(Error::UnsupportedWadFormat)?;

        let decode_config = general_purpose::GeneralPurposeConfig::new()
            .with_decode_padding_mode(bitcoin::base64::engine::DecodePaddingMode::Indifferent);
        let decoded = GeneralPurpose::new(&alphabet::URL_SAFE, decode_config).decode(s)?;
        let token = ciborium::from_reader(&decoded[..])?;
        Ok(token)
    }
}

impl<U: Unit + Serialize> fmt::Display for CompactWads<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tokens = Vec::new();
        for wad in &self.0 {
            tokens.push(wad.to_string());
        }
        write!(f, "{}", tokens.join(":"))
    }
}

impl<U: Unit + DeserializeOwned> FromStr for CompactWads<U> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // First, try to parse as colon-separated tokens (new format)
        if s.contains(':') {
            let token_strings: Vec<&str> = s.split(':').collect();

            let mut wads = Vec::with_capacity(token_strings.len());
            for token_str in token_strings {
                let wad = CompactWad::from_str(token_str)
                    .map_err(|e| Error::InvalidWadToken(Box::new(e)))?;
                wads.push(wad);
            }

            return Ok(CompactWads(wads));
        }

        // Try to parse as a single CompactWad (new format for single tokens)
        if s.starts_with(CASHU_PREFIX) {
            if let Ok(wad) = CompactWad::from_str(s) {
                return Ok(CompactWads(vec![wad]));
            }
        }

        // Fallback to old CBOR format for backward compatibility
        let s = s
            .strip_prefix(CASHU_PREFIX)
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

#[cfg(test)]
mod tests {
    use super::*;
    use nuts::nut00::secret::Secret;
    use nuts::nut01::PublicKey;
    use nuts::nut02::KeysetId;
    use nuts::{Amount, traits::Unit};
    use std::str::FromStr;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum TestUnit {
        Sat,
    }

    impl std::fmt::Display for TestUnit {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "sat")
        }
    }

    impl From<TestUnit> for u32 {
        fn from(_: TestUnit) -> Self {
            0
        }
    }

    impl FromStr for TestUnit {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "sat" => Ok(TestUnit::Sat),
                _ => Err("invalid unit"),
            }
        }
    }

    impl Unit for TestUnit {}

    impl AsRef<str> for TestUnit {
        fn as_ref(&self) -> &str {
            "sat"
        }
    }

    fn create_test_compact_wad(node_url: &str, amount: u64) -> CompactWad<TestUnit> {
        let keyset_id = KeysetId::from_bytes(&[0, 1, 2, 3, 4, 5, 6, 7]).unwrap();
        let secret =
            Secret::from_str("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let pubkey = PublicKey::from_slice(&[
            3, 23, 183, 225, 206, 31, 159, 148, 195, 42, 67, 115, 146, 41, 248, 140, 11, 3, 51, 41,
            111, 180, 110, 143, 114, 179, 192, 72, 147, 222, 233, 25, 52,
        ])
        .unwrap();

        // Use the correct scheme based on the tls feature
        let node_url = if cfg!(feature = "tls") {
            NodeUrl::from_str(&format!("https://{}", node_url)).unwrap()
        } else {
            NodeUrl::from_str(&format!("http://{}", node_url)).unwrap()
        };

        CompactWad {
            node_url,
            unit: TestUnit::Sat,
            memo: None,
            proofs: vec![CompactKeysetProofs {
                keyset_id,
                proofs: vec![CompactProof {
                    amount: Amount::from(amount),
                    secret,
                    c: pubkey,
                }],
            }],
        }
    }

    #[test]
    fn test_compact_wads_colon_separated_serialization() {
        let wad1 = create_test_compact_wad("mint1.example.com", 100);
        let wad2 = create_test_compact_wad("mint2.example.com", 200);
        let wads = CompactWads::new(vec![wad1, wad2]);

        let serialized = wads.to_string();

        // Should contain colon separator
        assert!(serialized.contains(':'));

        // Should have two cashuB prefixes
        let cashu_count = serialized.matches(CASHU_PREFIX).count();
        assert_eq!(cashu_count, 2);

        // Should be able to split into two tokens
        let parts: Vec<&str> = serialized.split(':').collect();
        assert_eq!(parts.len(), 2);

        // Each part should be a valid cashuB token
        for part in parts {
            assert!(part.starts_with(CASHU_PREFIX));
        }
    }

    #[test]
    fn test_compact_wads_colon_separated_deserialization() {
        let wad1 = create_test_compact_wad("mint1.example.com", 100);
        let wad2 = create_test_compact_wad("mint2.example.com", 200);
        let original_wads = CompactWads::new(vec![wad1, wad2]);

        let serialized = original_wads.to_string();
        let deserialized: CompactWads<TestUnit> = CompactWads::from_str(&serialized).unwrap();

        assert_eq!(original_wads.0.len(), deserialized.0.len());
        assert_eq!(original_wads.0[0].node_url, deserialized.0[0].node_url);
        assert_eq!(original_wads.0[1].node_url, deserialized.0[1].node_url);
    }

    #[test]
    fn test_compact_wads_single_token() {
        let wad = create_test_compact_wad("mint1.example.com", 100);
        let wads = CompactWads::new(vec![wad]);

        let serialized = wads.to_string();

        // Single token should not have colon
        assert!(!serialized.contains(':'));
        assert!(serialized.starts_with(CASHU_PREFIX));

        // Should round-trip correctly
        let deserialized: CompactWads<TestUnit> = CompactWads::from_str(&serialized).unwrap();
        assert_eq!(wads.0.len(), deserialized.0.len());
    }

    #[test]
    fn test_compact_wads_multiple_tokens() {
        let wad1 = create_test_compact_wad("mint1.example.com", 100);
        let wad2 = create_test_compact_wad("mint2.example.com", 200);
        let wad3 = create_test_compact_wad("mint3.example.com", 300);
        let wads = CompactWads::new(vec![wad1, wad2, wad3]);

        let serialized = wads.to_string();

        // Should have two colons (3 tokens)
        let colon_count = serialized.matches(':').count();
        assert_eq!(colon_count, 2);

        // Should have three cashuB prefixes
        let cashu_count = serialized.matches(CASHU_PREFIX).count();
        assert_eq!(cashu_count, 3);

        // Should round-trip correctly
        let deserialized: CompactWads<TestUnit> = CompactWads::from_str(&serialized).unwrap();
        assert_eq!(wads.0.len(), deserialized.0.len());
        assert_eq!(deserialized.0.len(), 3);
    }

    #[test]
    fn test_compact_wads_backward_compatibility() {
        // Create a CBOR-encoded multi-wad token (old format)
        let wad1 = create_test_compact_wad("mint1.example.com", 100);
        let wad2 = create_test_compact_wad("mint2.example.com", 200);
        let wads = CompactWads::new(vec![wad1, wad2]);

        // Manually create old CBOR format
        let mut data = Vec::new();
        ciborium::into_writer(&wads, &mut data).unwrap();
        let encoded = general_purpose::URL_SAFE.encode(data);
        let old_format = format!("{}{}", CASHU_PREFIX, encoded);

        // Should be able to parse old format
        let deserialized: CompactWads<TestUnit> = CompactWads::from_str(&old_format).unwrap();
        assert_eq!(wads.0.len(), deserialized.0.len());
    }

    #[test]
    fn test_compact_wads_invalid_token_in_colon_separated() {
        let invalid_colon_separated = "cashuBvalidtoken:invalidtoken";

        let result = CompactWads::<TestUnit>::from_str(invalid_colon_separated);
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::InvalidWadToken(_) => (),
            other => panic!("Expected InvalidWadToken error, got: {:?}", other),
        }
    }

    #[test]
    fn test_compact_wads_empty_input() {
        let result = CompactWads::<TestUnit>::from_str("");
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::UnsupportedWadFormat => (),
            other => panic!("Expected UnsupportedWadFormat error, got: {:?}", other),
        }
    }
}
