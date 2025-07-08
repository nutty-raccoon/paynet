use bitcoin::{hex::DisplayHex, Network, PrivateKey};
use bip39::{Mnemonic, Language};

pub fn create_seed_phrase(word_count: Option<u8>) -> Mnemonic {
    let mnemonic = Mnemonic::generate_in(Language::English,word_count.unwrap_or(12) as usize, ).expect("Failed to create mnemonic from entropy");
    mnemonic
}

pub fn derive_private_key(seed_phrase:&Mnemonic) -> PrivateKey {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");
    // Use BIP32 derivation path m/0'/0/0 to derive the first private key
    // This follows the standard Bitcoin derivation path for the first account, first external address
    // https://cashubtc.github.io/nuts/13
    let path = "m/0'/0/0";
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to create master key");
    let derivation_path: bitcoin::bip32::DerivationPath = path.parse().expect("Failed to parse derivation path");
    let derived_key = master_key.derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path).expect("Failed to derive private key");
    let private_key = PrivateKey::new(derived_key.private_key, Network::Bitcoin);
    private_key

}

pub fn derive_private_key_from_path(seed_phrase:&Mnemonic, path: &str) -> PrivateKey {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");
    // Use BIP32 derivation path m/0'/0/0 to derive the first private key
    // This follows the standard Bitcoin derivation path for the first account, first external address
    // https://cashubtc.github.io/nuts/13
    let master_key = bitcoin::bip32::Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("Failed to create master key");
    let derivation_path: bitcoin::bip32::DerivationPath = path.parse().expect("Failed to parse derivation path");
    let derived_key = master_key.derive_priv(&bitcoin::secp256k1::Secp256k1::new(), &derivation_path).expect("Failed to derive private key");
    let private_key = PrivateKey::new(derived_key.private_key, Network::Bitcoin);
    private_key

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_seed_phrase() {
        let seed_phrase = create_seed_phrase(None);
        println!("seed_phrase: {}", seed_phrase);
    }

    #[test]
    fn test_create_private_key() {
        let seed_phrase = create_seed_phrase(None);
        let private_key = derive_private_key(&seed_phrase);
        println!("private_key: {:?}", private_key);
    }
}
