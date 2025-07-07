use bitcoin::{Network, PrivateKey};
use bip39::{Mnemonic, Language};

pub fn create_seed_phrase(word_count: Option<u8>) -> Mnemonic {
    let mut rng = bitcoin::secp256k1::rand::thread_rng();
    let mnemonic = Mnemonic::generate_in(Language::English,word_count.unwrap_or(12) as usize, ).expect("Failed to create mnemonic from entropy");
    mnemonic
}

pub fn derive_private_key(seed_phrase:&Mnemonic) -> PrivateKey {
    // Convert mnemonic to seed using BIP39 standard (no passphrase)
    let seed = Mnemonic::to_seed_normalized(seed_phrase, "");
    
    // Use BIP32 derivation path m/0'/0/0 to derive the first private key
    // This follows the standard Bitcoin derivation path for the first account, first external address
    let path = "m/0'/0/0";
    let private_key = PrivateKey::from_slice(&seed, Network::Bitcoin).expect("Failed to derive private key from seed");
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
