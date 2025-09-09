use bip39::Mnemonic;
use bitcoin::bip32::Xpriv;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::db;

pub mod keyring;
#[cfg(feature = "sqlite-seed-phrase")]
pub mod sqlite;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Seed phrase not found in keyring")]
    SeedPhraseNotFound,
    #[error("Wallet already exists")]
    WalletAlreadyExists,
    #[error("seed phrase manager error: {0}")]
    SeedPhraseManager(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub trait SeedPhraseManager: Clone {
    type Error: std::error::Error + Send + Sync + 'static + From<crate::seed_phrase::Error>;

    fn store_seed_phrase(&self, seed_phrase: &Mnemonic) -> Result<(), Self::Error>;
    fn get_seed_phrase(&self) -> Result<Option<Mnemonic>, Self::Error>;

    fn get_private_key(&self) -> Result<Option<Xpriv>, Self::Error> {
        let opt_pk = match self.get_seed_phrase()? {
            Some(mnemonic) => Some(crate::seed_phrase::derive_private_key(&mnemonic)?),
            None => None,
        };

        Ok(opt_pk)
    }

    fn has_seed_phrase(&self) -> Result<bool, Self::Error> {
        Ok(self.get_seed_phrase()?.is_some())
    }
}

/// Restore a wallet from an existing seed phrase
/// This function stores the seed phrase in the keyring and creates a wallet record in the database
pub fn restore(
    seed_phrase_manager: impl SeedPhraseManager,
    db_conn: &Connection,
    seed_phrase: Mnemonic,
) -> Result<Option<Mnemonic>, Error> {
    // Check if wallet already exists in keyring
    let opt_previous_seed_phrase = if seed_phrase_manager
        .has_seed_phrase()
        .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?
    {
        let previous_seed_phrase = seed_phrase_manager
            .get_seed_phrase()
            .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?;
        match previous_seed_phrase {
            None => None,
            Some(mn) if mn == seed_phrase => None,
            Some(mn) => {
                seed_phrase_manager
                    .store_seed_phrase(&seed_phrase)
                    .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?;
                Some(mn)
            }
        }
    } else {
        // Store seed phrase in keyring (secure OS-level storage)
        seed_phrase_manager
            .store_seed_phrase(&seed_phrase)
            .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?;
        None
    };

    // Create wallet metadata record in database (without sensitive data)
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let wallet = db::wallet::Wallet {
        created_at: current_time,
        updated_at: current_time,
        is_restored: true,
    };

    db::wallet::create(db_conn, wallet)?;

    Ok(opt_previous_seed_phrase)
}

/// Initialize a new wallet with the provided seed phrase
/// This function stores the seed phrase in the keyring and creates a wallet record in the database
pub fn init(
    seed_phrase_manager: impl SeedPhraseManager,
    db_conn: &Connection,
    seed_phrase: &Mnemonic,
) -> Result<(), Error> {
    // Store seed phrase in keyring (secure OS-level storage)
    seed_phrase_manager
        .store_seed_phrase(seed_phrase)
        .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?;

    // Create wallet metadata record in database (without sensitive data)
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let wallet = db::wallet::Wallet {
        created_at: current_time,
        updated_at: current_time,
        is_restored: false,
    };

    db::wallet::create(db_conn, wallet)?;

    Ok(())
}

/// Check if a wallet exists
pub fn exists(seed_phrase_manager: impl SeedPhraseManager) -> Result<bool, Error> {
    Ok(seed_phrase_manager
        .get_seed_phrase()
        .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?
        .is_some())
}

/// Get the seed phrase from keyring
pub fn get_seed_phrase(seed_phrase_manager: impl SeedPhraseManager) -> Result<Mnemonic, Error> {
    seed_phrase_manager
        .get_seed_phrase()
        .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?
        .ok_or(Error::SeedPhraseNotFound)
}

/// Get the private key derived from the seed phrase stored in keyring
pub fn get_private_key(seed_phrase_manager: impl SeedPhraseManager) -> Result<Xpriv, Error> {
    seed_phrase_manager
        .get_private_key()
        .map_err(|e| Error::SeedPhraseManager(Box::new(e)))?
        .ok_or(Error::SeedPhraseNotFound)
}
