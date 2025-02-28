use std::fmt;
use std::str::FromStr;

use crate::Amount;
use crate::nut00::Proof;
use crate::nut01::PublicKey;
use crate::nut02::KeysetId;
use crate::traits::Unit;
use num_traits::CheckedAdd;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// use crate::mint_url::MintUrl;
use bitcoin::base64::engine::{GeneralPurpose, general_purpose};
use bitcoin::base64::{Engine as _, alphabet};

use super::secret::Secret;
use super::{Proofs, errors::Error};

/// Token V4
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenV4<U: Unit> {
    /// Mint Url
    #[serde(rename = "m")]
    pub mint_url: String,
    /// Token Unit
    #[serde(rename = "u")]
    pub unit: U,
    /// Memo for token
    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Proofs grouped by keyset_id
    #[serde(rename = "t")]
    pub token: Vec<TokenV4Token>,
}

impl<U: Unit> TokenV4<U> {
    /// Proofs from token
    pub fn proofs(&self) -> Proofs {
        self.token
            .iter()
            .flat_map(|token| token.proofs.iter().map(|p| p.proof(&token.keyset_id)))
            .collect()
    }

    /// Value
    #[inline]
    pub fn value(&self) -> Result<Amount, Error> {
        let mut sum = Amount::ZERO;
        for token in self.token.iter() {
            for proof in token.proofs.iter() {
                sum = sum.checked_add(&proof.amount).ok_or(Error::Overflow)?;
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

impl<U: Unit + Serialize> fmt::Display for TokenV4<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use serde::ser::Error;
        let mut data = Vec::new();
        ciborium::into_writer(self, &mut data).map_err(|e| fmt::Error::custom(e.to_string()))?;
        let encoded = general_purpose::URL_SAFE.encode(data);
        write!(f, "cashuB{}", encoded)
    }
}

impl<U: Unit + DeserializeOwned> FromStr for TokenV4<U> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix("cashuB").ok_or(Error::UnsupportedToken)?;

        let decode_config = general_purpose::GeneralPurposeConfig::new()
            .with_decode_padding_mode(bitcoin::base64::engine::DecodePaddingMode::Indifferent);
        let decoded = GeneralPurpose::new(&alphabet::URL_SAFE, decode_config).decode(s)?;
        let token: Self = ciborium::from_reader(&decoded[..])?;
        Ok(token)
    }
}

/// Token V4 Token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenV4Token {
    /// `Keyset id`
    #[serde(
        rename = "i",
        serialize_with = "serialize_v4_keyset_id",
        deserialize_with = "deserialize_v4_keyset_id"
    )]
    pub keyset_id: KeysetId,
    /// Proofs
    #[serde(rename = "p")]
    pub proofs: Vec<ProofV4>,
}

fn serialize_v4_keyset_id<S>(keyset_id: &KeysetId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&keyset_id.to_bytes())
}

fn deserialize_v4_keyset_id<'de, D>(deserializer: D) -> Result<KeysetId, D::Error>
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

impl TokenV4Token {
    /// Create new [`TokenV4Token`]
    pub fn new(keyset_id: KeysetId, proofs: Proofs) -> Self {
        Self {
            keyset_id,
            proofs: proofs.into_iter().map(|p| p.into()).collect(),
        }
    }
}

/// Proof V4
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofV4 {
    /// Amount in satoshi
    #[serde(rename = "a")]
    pub amount: Amount,
    /// Secret message
    #[serde(rename = "s")]
    pub secret: Secret,
    /// Unblinded signature
    #[serde(
        serialize_with = "serialize_v4_pubkey",
        deserialize_with = "deserialize_v4_pubkey"
    )]
    pub c: PublicKey,
}

impl ProofV4 {
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

impl From<Proof> for ProofV4 {
    fn from(proof: Proof) -> ProofV4 {
        let Proof {
            amount,
            keyset_id: _,
            secret,
            c,
        } = proof;
        ProofV4 { amount, secret, c }
    }
}

fn serialize_v4_pubkey<S>(key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&key.to_bytes())
}

fn deserialize_v4_pubkey<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let bytes = Vec::<u8>::deserialize(deserializer)?;
    PublicKey::from_slice(&bytes).map_err(serde::de::Error::custom)
}
