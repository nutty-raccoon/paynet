use bip39::Mnemonic;
use keyring::Entry;

use crate::seed_phrase;

const KEYRING_USER: &str = "seed_phrase";

#[derive(Debug, Copy, Clone)]
pub struct SeedPhraseManager {
    service: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to interact with keyring: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("failed to parse stored value: {0}")]
    SeedPhrase(#[from] crate::seed_phrase::Error),
}

impl SeedPhraseManager {
    pub const fn new(app_identifier: &'static str) -> Self {
        Self {
            service: app_identifier,
        }
    }
}

impl super::SeedPhraseManager for SeedPhraseManager {
    type Error = Error;

    fn store_seed_phrase(&self, seed_phrase: &Mnemonic) -> Result<(), Self::Error> {
        let entry = Entry::new(self.service, KEYRING_USER)?;
        entry.set_password(&seed_phrase.to_string())?;

        Ok(())
    }

    fn get_seed_phrase(&self) -> Result<Option<Mnemonic>, Self::Error> {
        let entry = Entry::new(self.service, KEYRING_USER)?;

        let seed_phrase_str = match entry.get_password() {
            Ok(s) => s,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(Self::Error::Keyring(e)),
        };

        let mnemonic = seed_phrase::create_from_str(&seed_phrase_str)?;

        Ok(Some(mnemonic))
    }
}
