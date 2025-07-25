use bip39::{Language, Mnemonic};
use bitcoin::{Network, PrivateKey, bip32::Xpriv};
use nuts::Amount;
use nuts::nut00::secret::Secret;
use nuts::nut01::PublicKey;
use nuts::nut02::KeysetId;
use nuts::{dhke::blind_message, nut00::BlindedMessage, nut01::SecretKey};
use std::collections::HashMap;
use std::str::FromStr;
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

// Use BIP32 derivation path m/0'/0/0 to derive the first private key
// This follows the standard Bitcoin derivation path for the first account, first external address
// https://cashubtc.github.io/nuts/13
// https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
pub fn derive_private_key(seed_phrase: &Mnemonic) -> Result<Xpriv, Error> {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");

    let path = "m/0'/0/0";
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)
        .map_err(|e| Error::MasterKey(e.to_string()))?;
    let derivation_path = path
        .parse::<bitcoin::bip32::DerivationPath>()
        .map_err(|e| Error::DerivationPath(e.to_string()))?;
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let derived_key = master_key
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| Error::DerivePriv(e.to_string()))?;

    Ok(derived_key)
}

pub fn derive_private_key_from_path(seed_phrase: &Mnemonic, path: &str) -> PrivateKey {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");
    // Use BIP32 derivation path m/0'/0/0 to derive the first private key
    // This follows the standard Bitcoin derivation path for the first account, first external address
    // https://cashubtc.github.io/nuts/13
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)
        .expect("Failed to create master key");
    let derivation_path: bitcoin::bip32::DerivationPath =
        path.parse().expect("Failed to parse derivation path");
    let derived_key = master_key
        .derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path)
        .expect("Failed to derive private key");

    PrivateKey::new(derived_key.private_key, Network::Bitcoin)
}

pub fn convert_private_key_to_xpriv(private_key: String) -> Result<Xpriv, Error> {
    let xpriv = Xpriv::from_str(&private_key)
        .map_err(|e| Error::MasterKey(format!("{}: {:?}", e, std::error::Error::source(&e))))?;
    Ok(xpriv)
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
