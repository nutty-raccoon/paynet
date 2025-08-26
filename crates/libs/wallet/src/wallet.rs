use bip39::Mnemonic;
use bitcoin::bip32::Xpriv;
use keyring::Entry;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::{db, seed_phrase};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
    #[error("Failed to access keyring: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("Seed phrase not found in keyring")]
    SeedPhraseNotFound,
    #[error("Failed to parse seed phrase: {0}")]
    ParseSeedPhrase(String),
    #[error("Wallet already exists")]
    WalletAlreadyExists,
    #[error("No wallet found")]
    NoWalletFound,
}

const KEYRING_USER: &str = "seed_phrase";

// Internal keyring functions - not exposed in public API
fn store_seed_phrase_in_keyring(service: &str, mnemonic: &Mnemonic) -> Result<(), Error> {
    let entry = Entry::new(service, KEYRING_USER)?;
    entry.set_password(&mnemonic.to_string())?;
    Ok(())
}

fn get_seed_phrase_from_keyring(service: &str) -> Result<Mnemonic, Error> {
    let entry = Entry::new(service, KEYRING_USER)?;

    let seed_phrase_str = entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => Error::SeedPhraseNotFound,
        _ => Error::Keyring(e),
    })?;

    seed_phrase::create_from_str(&seed_phrase_str)
        .map_err(|e| Error::ParseSeedPhrase(e.to_string()))
}

fn get_private_key_from_keyring(service: &str) -> Result<Xpriv, Error> {
    let mnemonic = get_seed_phrase_from_keyring(service)?;
    let xpriv = seed_phrase::derive_private_key(&mnemonic)?;
    Ok(xpriv)
}

fn has_seed_phrase_in_keyring(service: &str) -> Result<bool, Error> {
    match get_seed_phrase_from_keyring(service) {
        Ok(_) => Ok(true),
        Err(Error::SeedPhraseNotFound) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Restore a wallet from an existing seed phrase
/// This function stores the seed phrase in the keyring and creates a wallet record in the database
pub fn restore(
    app_identifier: &str,
    db_conn: &Connection,
    seed_phrase: Mnemonic,
) -> Result<Option<Mnemonic>, Error> {
    // Check if wallet already exists in database
    if db::wallet::count_wallets(db_conn)? > 0 {
        return Err(Error::WalletAlreadyExists);
    }

    // Check if wallet already exists in keyring
    let opt_previous_seed_phrase = if has_seed_phrase_in_keyring(app_identifier)? {
        let previous_seed_phrase = get_seed_phrase_from_keyring(app_identifier)?;
        if previous_seed_phrase == seed_phrase {
            None
        } else {
            store_seed_phrase_in_keyring(app_identifier, &seed_phrase)?;
            Some(previous_seed_phrase)
        }
    } else {
        // Store seed phrase in keyring (secure OS-level storage)
        store_seed_phrase_in_keyring(app_identifier, &seed_phrase)?;
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
    app_identifier: &str,
    db_conn: &Connection,
    seed_phrase: &Mnemonic,
) -> Result<(), Error> {
    // Check if wallet already exists in database
    if db::wallet::count_wallets(db_conn)? > 0 {
        return Err(Error::WalletAlreadyExists);
    }

    // Store seed phrase in keyring (secure OS-level storage)
    store_seed_phrase_in_keyring(app_identifier, seed_phrase)?;

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
pub fn exists(db_conn: &Connection) -> Result<bool, Error> {
    Ok(db::wallet::count_wallets(db_conn)? > 0)
}

/// Get the seed phrase from keyring
pub fn get_seed_phrase(app_identifier: &str) -> Result<Mnemonic, Error> {
    get_seed_phrase_from_keyring(app_identifier).map_err(|e| match e {
        Error::SeedPhraseNotFound => Error::NoWalletFound,
        _ => e,
    })
}

/// Get the private key derived from the seed phrase stored in keyring
pub fn get_private_key(app_identifier: &str) -> Result<Xpriv, Error> {
    get_private_key_from_keyring(app_identifier).map_err(|e| match e {
        Error::SeedPhraseNotFound => Error::NoWalletFound,
        _ => e,
    })
}
