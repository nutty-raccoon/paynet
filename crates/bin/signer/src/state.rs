use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bitcoin::{
    bip32::{ChildNumber, DerivationPath, Xpriv},
    key::Secp256k1,
};
use nuts::{
    nut01::SetKeyPairs,
    nut02::{KeysetId, MintKeySet},
    traits::Unit,
};

use crate::server_errors::Error;

#[derive(Debug, Clone)]
pub struct SharedRootKey(pub Arc<Xpriv>);

impl SharedRootKey {
    pub fn generate_keyset<U: Unit>(&self, unit: U, index: u32, max_order: u8) -> MintKeySet<U> {
        let unit_idx = unit.into();
        let secp_ctx = Secp256k1::new();

        let derivation_path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(0).expect("0 is a valid index"),
            ChildNumber::from_hardened_idx(unit_idx).expect("should be a valid index"),
            ChildNumber::from_hardened_idx(index).expect("should be a valid index"),
        ]);

        let xpriv = self
            .0
            .derive_priv(&secp_ctx, &derivation_path)
            .expect("RNG busted");

        MintKeySet::generate(&secp_ctx, xpriv, unit, max_order)
    }

    pub fn get_pubkey(&self) -> bitcoin::secp256k1::PublicKey {
        let secp256k1 = Secp256k1::new();
        let private_key = &self.0.private_key;
        private_key.public_key(&secp256k1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SharedKeySetCache(pub Arc<RwLock<HashMap<KeysetId, Arc<SetKeyPairs>>>>);

impl SharedKeySetCache {
    pub fn insert(&self, keyset_id: KeysetId, key_pairs: SetKeyPairs) -> Result<(), Error> {
        let mut write_lock = self.0.write().map_err(|_| Error::LockPoisoned)?;

        write_lock.insert(keyset_id, Arc::new(key_pairs));

        Ok(())
    }
}
