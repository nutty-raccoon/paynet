use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Amount, AmountStr, nut02::KeysetId};

use super::{mint_keys::SetKeyPairs, public_key::PublicKey};

/// Mint public keys per amount.
///
/// This is a variation of [MintKeys] that only exposes the public keys.
///
/// See [NUT-01]
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetPubKeys(BTreeMap<AmountStr, PublicKey>);

impl From<SetKeyPairs> for SetPubKeys {
    fn from(keys: SetKeyPairs) -> Self {
        Self(
            keys.0
                .into_iter()
                .map(|(amount, keypair)| (AmountStr::from(amount), keypair.public_key))
                .collect(),
        )
    }
}

impl SetPubKeys {
    /// Create new [`Keys`]
    #[inline]
    pub fn new(keys: BTreeMap<AmountStr, PublicKey>) -> Self {
        Self(keys)
    }

    /// Get [`Keys`]
    #[inline]
    pub fn keys(&self) -> &BTreeMap<AmountStr, PublicKey> {
        &self.0
    }

    /// Get [`PublicKey`] for [`Amount`]
    #[inline]
    pub fn amount_key(&self, amount: Amount) -> Option<PublicKey> {
        self.0.get(&AmountStr::from(amount)).copied()
    }

    /// Iterate through the (`Amount`, `PublicKey`) entries in the Map
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&AmountStr, &PublicKey)> {
        self.0.iter()
    }
}

impl From<&SetPubKeys> for KeysetId {
    fn from(keys: &SetPubKeys) -> Self {
        let iter = keys.iter().map(|(_, pubkey)| *pubkey);

        Self::from_iter(iter)
    }
}
