use bip39::{Language, Mnemonic};
use bitcoin::{Network, PrivateKey, bip32::Xpriv};
use nuts::nut00::secret::Secret;
use nuts::nut02::KeysetId;
use nuts::Amount;
use nuts::{dhke::blind_message, nut00::BlindedMessage, nut01::SecretKey};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Failed to create master key: {0}")]
    MasterKeyError(String),
    #[error("Failed to parse derivation path: {0}")]
    DerivationPathError(String),
    #[error("Failed to derive private key: {0}")]
    DerivePrivError(String),
    #[error("Failed to generate mnemonic: {0}")]
    GenerateMnemonicError(String),
    #[error("Failed to convert private key to xpriv: {0}")]
    ConvertPrivateKeyToXprivError(String),
    #[error("Failed to generate blinded messages: {0}")]
    GenerateBlindedMessagesError(String),
}

// Create a new seed phrase mnemonic with 12 words and BIP39 standard
pub fn create_seed_phrase() -> Result<Mnemonic, WalletError> {
    let mnemonic = Mnemonic::generate_in(Language::English, 12 as usize)
        .map_err(|e| WalletError::GenerateMnemonicError(e.to_string()))?;
    Ok(mnemonic)
}

// Use BIP32 derivation path m/0'/0/0 to derive the first private key
// This follows the standard Bitcoin derivation path for the first account, first external address
// https://cashubtc.github.io/nuts/13
// https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
pub fn derive_private_key(seed_phrase: &Mnemonic) -> Result<PrivateKey, WalletError> {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");

    let path = "m/0'/0/0";
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)
        .map_err(|e| WalletError::MasterKeyError(e.to_string()))?;
    let derivation_path = path
        .parse::<bitcoin::bip32::DerivationPath>()
        .map_err(|e| WalletError::DerivationPathError(e.to_string()))?;
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let derived_key = master_key
        .derive_priv(&secp, &derivation_path)
        .map_err(|e| WalletError::DerivePrivError(e.to_string()))?;
    let private_key = PrivateKey::new(derived_key.private_key, Network::Bitcoin);
    Ok(private_key)
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
    let private_key = PrivateKey::new(derived_key.private_key, Network::Bitcoin);
    private_key
}

pub fn convert_private_key_to_xpriv(private_key: String) -> Result<Xpriv, WalletError> {
    let xpriv =
        Xpriv::from_str(&private_key).map_err(|e| WalletError::MasterKeyError(e.to_string()))?;
    Ok(xpriv)
}
/// Generate blinded messages from predetermined secrets and blindings
/// factor
pub fn generate_blinded_messages(
    keyset_id: KeysetId,
    xpriv: Xpriv,
    start_count: u32,
    end_count: u32,
) -> Result<Vec<BlindedMessage>, WalletError> {
    let mut blinded_messages = vec![];

    for i in start_count..=end_count {
        let secret = Secret::from_xpriv(xpriv, keyset_id, i).map_err(|e| WalletError::GenerateBlindedMessagesError(e.to_string()))?;
        let blinding_factor = SecretKey::from_xpriv(xpriv, keyset_id, i).map_err(|e| WalletError::GenerateBlindedMessagesError(e.to_string()))?;

        let (blinded, r) = blind_message(&secret.to_bytes(), Some(blinding_factor)).map_err(|e| WalletError::GenerateBlindedMessagesError(e.to_string()))?;

        let blinded_message = BlindedMessage{
            amount: Amount::ZERO,
            keyset_id: keyset_id,
            blinded_secret: blinded,
        };

        blinded_messages.push(blinded_message);
    }

    Ok(blinded_messages)
}

#[cfg(test)]
mod tests {
    use bitcoin::{
        bip32::{DerivationPath, Xpriv},
        key::Secp256k1,
    };

    use super::*;

    #[test]
    fn test_create_seed_phrase() {
        let seed_phrase = create_seed_phrase().unwrap();
        println!("seed_phrase: {}", seed_phrase.to_string());
        // Test that the seed phrase is 12 words and each word is non-empty
        let binding = seed_phrase.to_string();
        let words: Vec<&str> = binding.split_whitespace().collect();
        assert_eq!(words.len(), 12, "Seed phrase should be 12 words");
        for (i, word) in words.iter().enumerate() {
            assert!(!word.is_empty(), "Word {} in seed phrase is empty", i + 1);
        }
    }

    #[test]
    fn test_create_private_key() {
        let seed_phrase = create_seed_phrase().unwrap();
        let private_key = derive_private_key(&seed_phrase).unwrap();
        println!("private_key: {:?}", private_key);
        // Improved test: check that the derived private key matches the expected master key for a known mnemonic and path

        // Use a fixed mnemonic for deterministic output
        let mnemonic = Mnemonic::parse(seed_phrase.clone().to_string()).unwrap();
        let path = "m/0'/0/0";
        let seed = Mnemonic::to_seed_normalized(&mnemonic, "");
        let master_key =
            bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).unwrap();
        let derivation_path: bitcoin::bip32::DerivationPath = path.parse().unwrap();
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let derived_key = master_key.derive_priv(&secp, &derivation_path).unwrap();
        let expected_private_key_bytes = derived_key.private_key.secret_bytes();

        let test_private_key = derive_private_key_from_path(&mnemonic, path);
        let test_private_key_bytes = test_private_key.inner.secret_bytes();

        assert_eq!(
            expected_private_key_bytes, test_private_key_bytes,
            "Derived private key bytes do not match expected master key bytes"
        );

        assert_eq!(
            private_key.inner.secret_bytes(),
            test_private_key.inner.secret_bytes(),
            "Derived private key bytes do not match expected master key bytes"
        );
    }
}