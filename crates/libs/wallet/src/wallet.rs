use bip39::Mnemonic;
use rusqlite::Connection;

use crate::{db, seed_phrase};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
}

pub fn restore(db_conn: &Connection, seed_phrase: Mnemonic) -> Result<(), Error> {
    let private_key = seed_phrase::derive_private_key(&seed_phrase)?;

    let wallet = db::wallet::Wallet {
        seed_phrase: seed_phrase.to_string(),
        private_key: private_key.to_string(),
        is_restored: true,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    db::wallet::create(db_conn, wallet)?;

    Ok(())
}

pub fn init(db_conn: &Connection, seed_phrase: &Mnemonic) -> Result<(), Error> {
    let private_key = seed_phrase::derive_private_key(seed_phrase)?;

    let wallet = db::wallet::Wallet {
        seed_phrase: seed_phrase.to_string(),
        private_key: private_key.to_string(),
        is_restored: false,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    db::wallet::create(db_conn, wallet)?;

    Ok(())
}
