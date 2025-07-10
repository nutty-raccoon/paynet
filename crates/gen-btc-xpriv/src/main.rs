use bitcoin::{NetworkKind, bip32::Xpriv};

fn main() {
    let seed: [u8; 32] = rand::random();

    let xpriv = Xpriv::new_master(NetworkKind::Test, &seed).unwrap();

    println!("{}", xpriv);
}
