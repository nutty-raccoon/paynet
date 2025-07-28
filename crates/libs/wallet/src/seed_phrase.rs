use bip39::{Language, Mnemonic};
use bitcoin::bip32::Xpriv;
use nuts::Amount;
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use nuts::nut02::KeysetId;
use nuts::{dhke::blind_message, nut00::BlindedMessage, nut01::SecretKey};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to create master key: {0}")]
    MasterKey(String),
    #[error("Failed to parse derivation path: {0}")]
    DerivationPath(String),
    #[error("Failed to derive private key: {0}")]
    DerivePriv(String),
    #[error("Failed to generate mnemonic: {0}")]
    GenerateMnemonic(String),
    #[error("Failed to convert private key to xpriv: {0}")]
    ConvertPrivateKeyToXpriv(String),
    #[error("Failed to generate blinded messages: {0}")]
    GenerateBlindedMessages(String),
}

// Create a new seed phrase mnemonic with 12 words and BIP39 standard
pub fn create_random() -> Result<Mnemonic, Error> {
    let mnemonic = Mnemonic::generate_in(Language::English, 12)
        .map_err(|e| Error::GenerateMnemonic(e.to_string()))?;
    Ok(mnemonic)
}

pub fn create_from_str(s: &str) -> Result<Mnemonic, Error> {
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, s)
        .map_err(|e| Error::GenerateMnemonic(e.to_string()))?;
    Ok(mnemonic)
}

pub fn derive_private_key(seed_phrase: &Mnemonic) -> Result<Xpriv, Error> {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");

    let master_key = Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)
        .map_err(|e| Error::MasterKey(e.to_string()))?;

    Ok(master_key)
}

/// Generate blinded messages from predetermined secrets and blindings
/// factor
#[allow(clippy::type_complexity)]
pub fn generate_blinded_messages(
    keyset_id: KeysetId,
    xpriv: Xpriv,
    start_count: u32,
    end_count: u32,
) -> Result<(Vec<BlindedMessage>, HashMap<PublicKey, (Secret, SecretKey)>), Error> {
    let n_bm = (end_count - start_count) as usize;
    let mut blinded_messages = Vec::with_capacity(n_bm);
    let mut secrets = HashMap::with_capacity(n_bm);

    for i in start_count..=end_count {
        let secret = Secret::from_xpriv(xpriv, keyset_id, i)
            .map_err(|e| Error::GenerateBlindedMessages(e.to_string()))?;
        let blinding_factor = SecretKey::from_xpriv(xpriv, keyset_id, i)
            .map_err(|e| Error::GenerateBlindedMessages(e.to_string()))?;

        let (blinded, r) = blind_message(&secret.to_bytes(), Some(blinding_factor))
            .map_err(|e| Error::GenerateBlindedMessages(e.to_string()))?;

        let blinded_message = BlindedMessage {
            amount: Amount::ZERO,
            keyset_id,
            blinded_secret: blinded,
        };

        blinded_messages.push(blinded_message);
        secrets.insert(blinded, (secret, r));
    }

    Ok((blinded_messages, secrets))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_seed_phrase() {
        let seed_phrase = create_random().unwrap();
        println!("seed_phrase: {}", seed_phrase);
        // Test that the seed phrase is 12 words and each word is non-empty
        let binding = seed_phrase.to_string();
        let words: Vec<&str> = binding.split_whitespace().collect();
        assert_eq!(words.len(), 12, "Seed phrase should be 12 words");
        for (i, word) in words.iter().enumerate() {
            assert!(!word.is_empty(), "Word {} in seed phrase is empty", i + 1);
        }
    }
}
