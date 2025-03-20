//! Secret

use std::fmt;
use std::str::FromStr;

use bitcoin::secp256k1::rand::{self, RngCore};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The secret data that allows spending ecash
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

/// Secret Errors
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid Length
    #[error("Invalid secret length: `{0}`")]
    InvalidLength(u64),
    /// Invalid Character
    #[error("Invalid hex character in secret")]
    InvalidHexCharacter,
    // /// Hex Error
    // #[error(transparent)]
    // Hex(#[from] hex::Error),
    /// Serde Json error
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

impl Default for Secret {
    fn default() -> Self {
        Self::generate()
    }
}

impl Secret {
    /// Create new [`Secret`]
    ///
    /// The secret must be a valid 64-character hex string representing
    /// a 32-byte value
    #[inline]
    pub fn new<S>(secret: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let s = secret.into();
        Self::validate(&s)?;
        Ok(Self(s))
    }

    /// Create a new Secret without validation
    ///
    /// # Safety
    ///
    /// This function should only be used in contexts where the input
    /// is guaranteed to be a valid 64-character hex string.
    #[inline]
    pub(crate) fn new_unchecked<S>(secret: S) -> Self
    where
        S: Into<String>,
    {
        Self(secret.into())
    }

    /// Validate that a string is a proper Secret
    fn validate(s: &str) -> Result<(), Error> {
        // Check the length
        if s.len() != 64 {
            return Err(Error::InvalidLength(s.len() as u64));
        }

        // Check that all characters are valid hex
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidHexCharacter);
        }

        Ok(())
    }

    /// Create secret value
    /// Generate a new random secret as the recommended 32 byte hex
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        let mut random_bytes = [0u8; 32];

        // Generate random bytes
        rng.fill_bytes(&mut random_bytes);
        // The secret string is hex encoded
        let secret = hex::encode(random_bytes);
        Self::new_unchecked(secret)
    }

    /// [`Secret`] as bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// [`Secret`] to bytes
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl FromStr for Secret {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<[u8]> for Secret {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Secret> for Vec<u8> {
    fn from(value: Secret) -> Vec<u8> {
        value.to_bytes()
    }
}

impl From<&Secret> for Vec<u8> {
    fn from(value: &Secret) -> Vec<u8> {
        value.to_bytes()
    }
}

impl AsRef<str> for Secret {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::assert_eq;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_secret_from_str() {
        let secret = Secret::generate();

        let secret_str = secret.to_string();

        assert_eq!(hex::decode(secret_str.clone()).unwrap().len(), 32);

        let secret_n = Secret::from_str(&secret_str).unwrap();

        assert_eq!(secret_n, secret)
    }

    #[test]
    fn test_secret_validation() {
        // Valid secret
        let valid_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        assert!(Secret::new(valid_hex).is_ok());
        assert!(Secret::from_str(valid_hex).is_ok());

        // Invalid length
        let too_short = "0123456789abcdef";
        let too_long = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef00";
        assert!(Secret::new(too_short).is_err());
        assert!(Secret::new(too_long).is_err());
        assert!(Secret::from_str(too_short).is_err());
        assert!(Secret::from_str(too_long).is_err());

        // Invalid characters
        let invalid_chars = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg";
        assert!(Secret::new(invalid_chars).is_err());
        assert!(Secret::from_str(invalid_chars).is_err());
    }
}
