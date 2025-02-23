use bitcoin::hashes::Hash;
use bitcoin::hashes::sha256::Hash as Sha256;
use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::nut01::PublicKey;

use super::Error;

/// Keyset version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeySetVersion {
    /// Current Version 00
    Version00,
}

impl TryFrom<u8> for KeySetVersion {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(Self::Version00),
            _ => Err(Error::UnknownVersion),
        }
    }
}

impl From<KeySetVersion> for u8 {
    fn from(_: KeySetVersion) -> Self {
        0
    }
}

impl fmt::Display for KeySetVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeySetVersion::Version00 => f.write_str("00"),
        }
    }
}

/// A keyset ID is an identifier for a specific keyset. It can be derived by
/// anyone who knows the set of public keys of a mint. The keyset ID **CAN**
/// be stored in a Cashu token such that the token can be used to identify
/// which mint or keyset it was generated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
#[serde(rename = "id")]
pub struct KeysetId {
    version: KeySetVersion,
    id: [u8; Self::BYTELEN],
}

impl KeysetId {
    pub const BYTELEN: usize = 7;
    pub const STRLEN: usize = Self::BYTELEN * 2;

    pub fn new(version: KeySetVersion, id: [u8; Self::BYTELEN]) -> Self {
        Self { version, id }
    }

    pub fn version(&self) -> KeySetVersion {
        self.version
    }

    pub fn id(&self) -> [u8; Self::BYTELEN] {
        self.id
    }

    /// [`Id`] to bytes
    pub fn to_bytes(&self) -> [u8; Self::BYTELEN + 1] {
        let mut bytes = [0; Self::BYTELEN + 1];

        bytes[0] = self.version.into();
        bytes[1..].copy_from_slice(&self.id);

        bytes
    }

    pub fn as_i64(&self) -> i64 {
        i64::from_be_bytes(self.to_bytes())
    }

    /// [`Id`] from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            version: KeySetVersion::try_from(bytes[0])?,
            id: bytes[1..].try_into()?,
        })
    }
}

/// As per NUT-02:
///   1. sort public keys by their amount in ascending order
///   2. concatenate all public keys to one string
///   3. HASH_SHA256 the concatenated public keys
///   4. take the first 14 characters of the hex-encoded hash
///   5. prefix it with a keyset ID version byte
impl FromIterator<PublicKey> for KeysetId {
    fn from_iter<T: IntoIterator<Item = PublicKey>>(iter: T) -> Self {
        let hash = Sha256::hash_byte_chunks(iter.into_iter().map(|pk| pk.to_bytes()));
        let hex_of_hash = hex::encode(hash.to_byte_array());

        Self::new(
            KeySetVersion::Version00,
            hex::decode(&hex_of_hash[0..Self::STRLEN])
                .expect("Keys hash could not be hex decoded")
                .try_into()
                .expect("Invalid length of hex id"),
        )
    }
}

// Used to generate a compressed unique identifier as part of the NUT13 spec
// This is a one-way function
impl From<KeysetId> for u32 {
    fn from(value: KeysetId) -> Self {
        let hex_bytes: [u8; 8] = value.to_bytes();

        let int = u64::from_be_bytes(hex_bytes);

        (int % (2_u64.pow(31) - 1)) as u32
    }
}

impl fmt::Display for KeysetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}{}", self.version, hex::encode(self.id)))
    }
}

impl FromStr for KeysetId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 16 {
            return Err(Error::Length);
        }

        let version = hex::decode(&s[..2])?[0].try_into()?;
        let id = hex::decode(&s[2..])?
            .try_into()
            .map_err(|_| Error::Length)?;

        Ok(Self { version, id })
    }
}

impl From<KeysetId> for String {
    fn from(value: KeysetId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for KeysetId {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl TryFrom<i64> for KeysetId {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let bytes = value.to_be_bytes();
        Self::from_bytes(&bytes)
    }
}

impl From<KeysetId> for i64 {
    fn from(value: KeysetId) -> Self {
        value.as_i64()
    }
}
