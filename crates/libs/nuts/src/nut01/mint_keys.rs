use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::Amount;

use super::{public_key::PublicKey, secret_key::SecretKey};

/// Mint key pairs per amount
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetKeyPairs(pub(crate) BTreeMap<Amount, KeyPair>);

impl Deref for SetKeyPairs {
    type Target = BTreeMap<Amount, KeyPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SetKeyPairs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SetKeyPairs {
    /// Create new [`MintKeys`]
    #[inline]
    pub fn new(map: BTreeMap<Amount, KeyPair>) -> Self {
        Self(map)
    }
}

/// Mint Public Private key pair
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPair {
    /// Publickey
    pub public_key: PublicKey,
    /// Secretkey
    pub secret_key: SecretKey,
}

impl KeyPair {
    /// [`MintKeyPair`] from secret key
    #[inline]
    pub fn from_secret_key(secret_key: SecretKey) -> Self {
        Self {
            public_key: secret_key.public_key(),
            secret_key,
        }
    }
}
