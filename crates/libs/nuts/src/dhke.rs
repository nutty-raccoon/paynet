//! Diffie-Hellmann key exchange

use std::ops::Deref;

use bitcoin::hashes::Hash;
use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::secp256k1::{
    Parity, PublicKey as NormalizedPublicKey, Scalar, Secp256k1, XOnlyPublicKey,
};
use thiserror::Error;

use crate::SECP256K1;
use crate::nut00::secret::Secret;
use crate::nut00::{BlindSignature, Proof};
use crate::nut01::PublicKey;
use crate::nut01::SecretKey;

const DOMAIN_SEPARATOR: &[u8; 28] = b"Secp256k1_HashToCurve_Cashu_";

/// NUT00 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Token could not be validated
    #[error("Token not verified")]
    TokenNotVerified,
    /// No valid point on curve
    #[error("No valid point found")]
    NoValidPoint,
    /// Secp256k1 error
    #[error(transparent)]
    Secp256k1(#[from] bitcoin::secp256k1::Error),
    #[error("Could not get proof")]
    CoudNotGetProof,
    #[error("Lengths of promises, rs, and secrets must be equal")]
    DifferentLength,
}

/// Deterministically maps a message to a public key point on the secp256k1
/// curve, utilizing a domain separator to ensure uniqueness.
///
/// For definationn in NUT see [NUT-00](https://github.com/cashubtc/nuts/blob/main/00.md)
pub fn hash_to_curve(message: &[u8]) -> Result<PublicKey, Error> {
    let msg_to_hash: Vec<u8> = [DOMAIN_SEPARATOR, message].concat();

    let msg_hash: [u8; 32] = Sha256Hash::hash(&msg_to_hash).to_byte_array();

    let mut counter: u32 = 0;
    while counter < 2_u32.pow(16) {
        let mut bytes_to_hash: Vec<u8> = Vec::with_capacity(36);
        bytes_to_hash.extend_from_slice(&msg_hash);
        bytes_to_hash.extend_from_slice(&counter.to_le_bytes());
        let hash: [u8; 32] = Sha256Hash::hash(&bytes_to_hash).to_byte_array();

        // Try to parse public key
        match XOnlyPublicKey::from_slice(&hash) {
            Ok(pk) => {
                return Ok(NormalizedPublicKey::from_x_only_public_key(pk, Parity::Even).into());
            }
            Err(_) => {
                counter += 1;
            }
        }
    }

    Err(Error::NoValidPoint)
}

/// Convert iterator of [`PublicKey`] to byte array
pub fn hash_e<I>(public_keys: I) -> [u8; 32]
where
    I: IntoIterator<Item = PublicKey>,
{
    let mut e: String = String::new();

    for public_key in public_keys.into_iter() {
        let uncompressed: [u8; 65] = public_key.to_uncompressed_bytes();
        e.push_str(&hex::encode(uncompressed));
    }

    Sha256Hash::hash(e.as_bytes()).to_byte_array()
}

/// Blind Message
///
/// `B_ = Y + rG`
pub fn blind_message(
    secret: &[u8],
    blinding_factor: Option<SecretKey>,
) -> Result<(PublicKey, SecretKey), Error> {
    let y: PublicKey = hash_to_curve(secret)?;
    let r: SecretKey = blinding_factor.unwrap_or_else(SecretKey::generate);
    Ok((y.combine(&r.public_key())?.into(), r))
}

/// Unblind Message
///
/// `C_ - rK`
pub fn unblind_message(
    // C_
    blind_key: &PublicKey,
    r: &SecretKey,
    // K
    mint_pubkey: &PublicKey,
) -> Result<PublicKey, Error> {
    let r: Scalar = Scalar::from(r.deref().to_owned());

    // a = r * K
    let a: PublicKey = mint_pubkey.mul_tweak(&SECP256K1, &r)?.into();

    // C_ - a
    let a: PublicKey = a.negate(&SECP256K1).into();
    Ok(blind_key.combine(&a)?.into()) // C_ + (-a)
}

/// Construct Proofs
#[cfg(feature = "nut12")]
pub fn construct_proofs(
    promises: Vec<BlindSignature>,
    rs: Vec<SecretKey>, // blinding factors
    secrets: Vec<Secret>,
    keys: &std::collections::HashMap<u64, PublicKey>,
) -> Result<Vec<Proof>, Error> {
    if promises.len() != rs.len() || promises.len() != secrets.len() {
        return Err(Error::DifferentLength);
    }

    promises
        .iter()
        .zip(rs.iter())
        .zip(secrets.iter())
        .map(|((promise, r), secret)| {
            // This part handles the DLEQ proof, adding the blinding factor `r`
            // which is required for Carol's verification scenario.
            let mut dleq = promise.dleq.clone();
            if let Some(dleq_proof) = &mut dleq {
                // The blinding factor `r` is added here. It was part of the BlindSignature
                // from the mint, but the `r` value itself was only known by Alice.
                dleq_proof.r = Some(hex::encode(r.to_secret_bytes()));
            }

            // Here we use your `unblind_message` function.
            // It takes the blinded signature from the promise (C_), the blinding
            // factor (r), and the mint's public key for that amount (K).
            let c = unblind_message(
                &promise.c,
                r,
                &keys[&(promise.amount.into_i64_repr() as u64)],
            )?;

            let proof = Proof {
                amount: promise.amount,
                // The KeysetId from the promise is carried over to the proof
                keyset_id: promise.keyset_id,
                secret: secret.clone(),
                // This is the final unblinded signature, ready for spending
                c,
                // The DLEQ proof, now including `r`, is attached to the final proof
                dleq,
            };
            Ok(proof)
        })
        .collect()
}

/// Sign Blind Message
///
/// `C_ = k * B_`, where:
/// * `k` is the private key of mint (one for each amount)
/// * `B_` is the blind message
#[inline]
pub fn sign_message(k: &SecretKey, blind_message: &PublicKey) -> Result<PublicKey, Error> {
    let k: Scalar = Scalar::from(k.deref().to_owned());
    Ok(blind_message.mul_tweak(&SECP256K1, &k)?.into())
}

/// Verify Message
pub fn verify_message(
    a: &SecretKey,
    unblind_message: PublicKey,
    msg: &[u8],
) -> Result<bool, Error> {
    // Y
    let y: PublicKey = hash_to_curve(msg)?;

    // Compute the expected unblind message
    let expected_unblind_message: PublicKey = y
        .mul_tweak(&Secp256k1::new(), &Scalar::from(*a.deref()))?
        .into();

    // Compare the unblind_message with the expected value
    Ok(unblind_message == expected_unblind_message)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_hash_to_curve() {
        let secret = "0000000000000000000000000000000000000000000000000000000000000000";
        let sec_hex = hex::decode(secret).unwrap();

        let y = hash_to_curve(&sec_hex).unwrap();
        let expected_y = PublicKey::from_hex(
            "024cce997d3b518f739663b757deaec95bcd9473c30a14ac2fd04023a739d1a725",
        )
        .unwrap();
        assert_eq!(y, expected_y);

        let secret = "0000000000000000000000000000000000000000000000000000000000000001";
        let sec_hex = hex::decode(secret).unwrap();
        let y = hash_to_curve(&sec_hex).unwrap();
        let expected_y = PublicKey::from_hex(
            "022e7158e11c9506f1aa4248bf531298daa7febd6194f003edcd9b93ade6253acf",
        )
        .unwrap();
        assert_eq!(y, expected_y);
        // Note that this message will take a few iterations of the loop before finding
        // a valid point
        let secret = "0000000000000000000000000000000000000000000000000000000000000002";
        let sec_hex = hex::decode(secret).unwrap();
        let y = hash_to_curve(&sec_hex).unwrap();
        let expected_y = PublicKey::from_hex(
            "026cdbe15362df59cd1dd3c9c11de8aedac2106eca69236ecd9fbe117af897be4f",
        )
        .unwrap();
        assert_eq!(y, expected_y);
    }

    #[test]
    fn test_hash_e() {
        let c = PublicKey::from_str(
            "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
        )
        .unwrap();

        let k = PublicKey::from_str(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let r1 = PublicKey::from_str(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let r2 = PublicKey::from_str(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let e = hash_e(vec![r1, r2, k, c]);
        let e_hex = hex::encode(e);

        assert_eq!(
            "a4dc034b74338c28c6bc3ea49731f2a24440fc7c4affc08b31a93fc9fbe6401e",
            e_hex
        )
    }

    #[test]
    fn test_blind_message() {
        let message =
            hex::decode("d341ee4871f1f889041e63cf0d3823c713eea6aff01e80f1719f08f9e5be98f6")
                .unwrap();
        let sec: SecretKey =
            SecretKey::from_hex("99fce58439fc37412ab3468b73db0569322588f62fb3a49182d67e23d877824a")
                .unwrap();

        let (b, r) = blind_message(&message, Some(sec.clone())).unwrap();

        assert_eq!(sec, r);
        assert_eq!(
            b,
            PublicKey::from_hex(
                "033b1a9737a40cc3fd9b6af4b723632b76a67a36782596304612a6c2bfb5197e6d"
            )
            .unwrap()
        );

        let message =
            hex::decode("f1aaf16c2239746f369572c0784d9dd3d032d952c2d992175873fb58fae31a60")
                .unwrap();
        let sec: SecretKey =
            SecretKey::from_hex("f78476ea7cc9ade20f9e05e58a804cf19533f03ea805ece5fee88c8e2874ba50")
                .unwrap();

        let (b, r) = blind_message(&message, Some(sec.clone())).unwrap();

        assert_eq!(sec, r);
        assert_eq!(
            b,
            PublicKey::from_hex(
                "029bdf2d716ee366eddf599ba252786c1033f47e230248a4612a5670ab931f1763"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_unblind_message() {
        let blind_key = PublicKey::from_hex(
            "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
        )
        .unwrap();

        let r =
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let a = PublicKey::from_hex(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let unblind = unblind_message(&blind_key, &r, &a).unwrap();

        assert_eq!(
            PublicKey::from_hex(
                "03c724d7e6a5443b39ac8acf11f40420adc4f99a02e7cc1b57703d9391f6d129cd"
            )
            .unwrap(),
            unblind
        );
    }

    #[test]
    fn test_sign_message() {
        use super::*;
        let message = "test_message";
        let sec =
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let (blind_message, _r) = blind_message(message.as_bytes(), Some(sec)).unwrap();
        // A
        let bob_sec =
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        // C_
        let signed = sign_message(&bob_sec, &blind_message).unwrap();

        assert_eq!(
            signed,
            PublicKey::from_hex(
                "025cc16fe33b953e2ace39653efb3e7a7049711ae1d8a2f7a9108753f1cdea742b"
            )
            .unwrap()
        );

        // A
        let bob_sec =
            SecretKey::from_hex("7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f")
                .unwrap();

        // C_
        let signed = sign_message(&bob_sec, &blind_message).unwrap();

        assert_eq!(
            signed,
            PublicKey::from_hex(
                "027726f0e5757b4202a27198369a3477a17bc275b7529da518fc7cb4a1d927cc0d"
            )
            .unwrap()
        );
    }

    #[test]
    fn test_full_bhke() {
        let message =
            hex::decode("d341ee4871f1f889041e63cf0d3823c713eea6aff01e80f1719f08f9e5be98f6")
                .unwrap();
        let alice_sec: SecretKey =
            SecretKey::from_hex("99fce58439fc37412ab3468b73db0569322588f62fb3a49182d67e23d877824a")
                .unwrap();

        let (b, r) = blind_message(&message, Some(alice_sec.clone())).unwrap();

        let bob_sec =
            SecretKey::from_hex("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        // C_
        let signed = sign_message(&bob_sec, &b).unwrap();

        let unblind = unblind_message(&signed, &r, &bob_sec.public_key()).unwrap();

        assert!(verify_message(&bob_sec, unblind, &message).is_ok());
    }
}
