//! NUT-12: Offline ecash signature validation (DLEQ proofs)
// https://cashubtc.github.io/nuts/12/

use bitcoin::hashes::{Hash, HashEngine, sha256::Hash as Sha256Hash};
use bitcoin::secp256k1::{PublicKey as SecpPublicKey, Scalar, SecretKey as SecpSecretKey};

use crate::SECP256K1;
use crate::dhke::{self, Error};
use crate::nut01::{PublicKey, SecretKey};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DleqProof {
    pub e: String,
    pub s: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
}

/// Generate DLEQ proof for blind signature (Mint side).
pub fn generate_dleq_proof(
    mint_private_key: &SecretKey,
    mint_public_key: &PublicKey,
    blinded_message: &PublicKey,
    blind_signature: &PublicKey,
) -> Result<(String, String), Error> {
    // Get `a` as a secret key
    let a_sk = SecpSecretKey::from_slice(mint_private_key.as_secret_bytes())?;

    // Generate deterministic nonce `r`
    let mut engine = Sha256Hash::engine();
    engine.input(&mint_private_key.to_secret_bytes());
    engine.input(&blinded_message.to_bytes());
    let r_hash = Sha256Hash::from_engine(engine);
    let r_scalar = Scalar::from_be_bytes(r_hash.to_byte_array()).expect("Hash output is valid");

    // Start with s = r. `s_sk` will be mutated to become the final `s`.
    let s_sk = SecpSecretKey::from_slice(&r_scalar.to_be_bytes())?;

    // Calculate R1 = rG and R2 = rB'
    let r1 = PublicKey::from(SecpPublicKey::from_secret_key(&SECP256K1, &s_sk));
    let r2 = PublicKey::from(blinded_message.mul_tweak(&SECP256K1, &r_scalar)?);

    // Calculate e = H(R1, R2, A, C')
    let e_bytes = dhke::hash_e([r1, r2, *mint_public_key, *blind_signature]);
    let e_scalar = Scalar::from_be_bytes(e_bytes).expect("Hash output is valid");

    // Calculate s = r + e*a
    // 1. Calculate `e*a` by tweaking `a_sk` in-place.
    let a_sk = a_sk.mul_tweak(&e_scalar)?;
    let ea_scalar = Scalar::from_be_bytes(a_sk.secret_bytes()).expect("Valid scalar");

    // 2. Add the result to `s_sk` (which is `r`) in-place. `s_sk` now holds `r + e*a`.
    let s_sk = s_sk.add_tweak(&ea_scalar)?;

    let s_scalar = Scalar::from_be_bytes(s_sk.secret_bytes()).expect("Valid scalar");

    Ok((
        hex::encode(e_scalar.to_be_bytes()),
        hex::encode(s_scalar.to_be_bytes()),
    ))
}

/// Verify DLEQ proof during minting/swapping
/// Alice verifies the mint's signature on her blinded message
pub fn verify_dleq_proof_alice(
    e_hex: &str,
    s_hex: &str,
    mint_public_key: &PublicKey,
    blinded_message: &PublicKey,
    blind_signature: &PublicKey,
) -> Result<bool, Error> {
    let e_scalar = Scalar::from_be_bytes(
        hex::decode(e_hex)?
            .try_into()
            .map_err(|_| Error::InvalidDleqProof)?,
    )?;
    let s_scalar = Scalar::from_be_bytes(
        hex::decode(s_hex)?
            .try_into()
            .map_err(|_| Error::InvalidDleqProof)?,
    )?;

    // R1 = sG - eA
    let s_sk = SecpSecretKey::from_slice(&s_scalar.to_be_bytes())?;
    let s_g = SecpPublicKey::from_secret_key(&SECP256K1, &s_sk);
    let e_a = mint_public_key.mul_tweak(&SECP256K1, &e_scalar)?;
    let e_a_neg = e_a.negate(&SECP256K1);
    let r1_computed = PublicKey::from(s_g.combine(&e_a_neg)?);

    // R2 = sB' - eC'
    let s_b_prime = blinded_message.mul_tweak(&SECP256K1, &s_scalar)?;
    let e_c_prime = blind_signature.mul_tweak(&SECP256K1, &e_scalar)?;
    let e_c_prime_neg = e_c_prime.negate(&SECP256K1);
    let r2_computed = PublicKey::from(s_b_prime.combine(&e_c_prime_neg)?);

    let e_computed_bytes =
        dhke::hash_e([r1_computed, r2_computed, *mint_public_key, *blind_signature]);

    // Return Ok with the boolean result of the comparison.
    Ok(e_scalar.to_be_bytes() == e_computed_bytes)
}

/// Verify DLEQ proof for a received token (Carol's case).
pub fn verify_dleq_proof_carol(
    e_hex: &str,
    s_hex: &str,
    r_hex: &str,
    mint_public_key: &PublicKey, // A
    secret: &[u8],
    unblinded_signature: &PublicKey, // C
) -> Result<bool, Error> {
    let r_scalar = Scalar::from_be_bytes(
        hex::decode(r_hex)?
            .try_into()
            .map_err(|_| Error::InvalidDleqProof)?,
    )?;

    // Reconstruct B' = Y + rG
    let y = dhke::hash_to_curve(secret)?;
    let r_sk = SecpSecretKey::from_slice(&r_scalar.to_be_bytes())?;
    let r_g = SecpPublicKey::from_secret_key(&SECP256K1, &r_sk);
    let blinded_message = PublicKey::from(y.combine(&r_g)?);

    // Reconstruct C' = C + rA
    let r_a = mint_public_key.mul_tweak(&SECP256K1, &r_scalar)?;
    let blinded_signature = PublicKey::from(unblinded_signature.combine(&r_a)?);

    verify_dleq_proof_alice(
        e_hex,
        s_hex,
        mint_public_key,
        &blinded_message,
        &blinded_signature,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use crate::{
        Amount,
        dhke::{self, construct_proofs, sign_message, unblind_message},
        nut00::{BlindSignature, secret::Secret},
        nut01::{PublicKey, SecretKey},
        nut02::KeysetId,
        nut12::{DleqProof, generate_dleq_proof, verify_dleq_proof_alice, verify_dleq_proof_carol},
    };

    // Helper function
    fn setup_mint_keys() -> (SecretKey, PublicKey) {
        let mint_private_key = SecretKey::generate();
        let mint_public_key = mint_private_key.public_key();
        (mint_private_key, mint_public_key)
    }

    #[test]
    fn test_dleq_full_roundtrip() {
        let (mint_private_key, mint_public_key) = setup_mint_keys();
        let secret = Secret::new("00".repeat(32)).unwrap();
        let secret_bytes = hex::decode(AsRef::<str>::as_ref(&secret)).unwrap();
        let (blind_message, blinding_factor_r) = dhke::blind_message(&secret_bytes, None).unwrap();
        let blinded_signature_c_prime = sign_message(&mint_private_key, &blind_message).unwrap();

        let (e, s) = generate_dleq_proof(
            &mint_private_key,
            &mint_public_key,
            &blind_message,
            &blinded_signature_c_prime,
        )
        .unwrap();

        let alice_verification = verify_dleq_proof_alice(
            &e,
            &s,
            &mint_public_key,
            &blind_message,
            &blinded_signature_c_prime,
        );
        assert!(
            alice_verification.is_ok(),
            "Alice's DLEQ verification should pass but failed: {:?}",
            alice_verification.err()
        );

        let unblinded_signature_c = unblind_message(
            &blinded_signature_c_prime,
            &blinding_factor_r,
            &mint_public_key,
        )
        .unwrap();

        let carol_verification = verify_dleq_proof_carol(
            &e,
            &s,
            &hex::encode(blinding_factor_r.to_secret_bytes()),
            &mint_public_key,
            &secret_bytes,
            &unblinded_signature_c,
        );
        assert!(
            carol_verification.is_ok(),
            "Carol's DLEQ verification should pass but failed: {:?}",
            carol_verification.err()
        );
    }

    #[test]
    fn test_construct_proofs_with_dleq() {
        let (mint_private_key, mint_public_key) = setup_mint_keys();
        let secret = Secret::new("11".repeat(32)).unwrap();
        let secret_bytes = hex::decode(AsRef::<str>::as_ref(&secret)).unwrap();
        let keyset_id = KeysetId::from_str("009a1f29b252a2b7").unwrap();
        let amount = Amount::from(100_u64);
        let (blind_message, blinding_factor_r) = dhke::blind_message(&secret_bytes, None).unwrap();
        let blinded_signature_c_prime = sign_message(&mint_private_key, &blind_message).unwrap();
        let (s, e) = generate_dleq_proof(
            &mint_private_key,
            &mint_public_key,
            &blind_message,
            &blinded_signature_c_prime,
        )
        .unwrap();
        let blind_signature = BlindSignature {
            amount,
            keyset_id,
            c: blinded_signature_c_prime,
            dleq: Some(DleqProof { e, s, r: None }),
        };
        let mut keys = HashMap::new();
        keys.insert(amount.into_i64_repr() as u64, mint_public_key);
        let proofs = construct_proofs(
            vec![blind_signature],
            vec![blinding_factor_r.clone()],
            vec![secret],
            &keys,
        )
        .unwrap();
        assert_eq!(proofs.len(), 1);
        let final_dleq = proofs[0].dleq.as_ref().unwrap();
        assert_eq!(
            final_dleq.r.as_ref().unwrap(),
            &hex::encode(blinding_factor_r.to_secret_bytes())
        );
    }

    #[test]
    fn test_nut12_hash_e_vector() {
        let r1 = PublicKey::from_hex(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let r2 = PublicKey::from_hex(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let k = PublicKey::from_hex(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        let c_prime = PublicKey::from_hex(
            "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
        )
        .unwrap();
        let expected_hash = "a4dc034b74338c28c6bc3ea49731f2a24440fc7c4affc08b31a93fc9fbe6401e";

        let result_bytes = dhke::hash_e([r1, r2, k, c_prime]);
        let result_hex = hex::encode(result_bytes);

        assert_eq!(
            result_hex, expected_hash,
            "hash_e did not produce the expected hash from the test vector"
        );
    }

    #[test]
    fn test_nut12_alice_verification_vector() {
        // This test uses the vector for "DLEQ verification on BlindSignature".
        // This is Alice's use case, where she verifies the signature from the mint.
        let mint_pubkey_a = PublicKey::from_hex(
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        )
        .unwrap();
        let blinded_message_b = PublicKey::from_hex(
            "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
        )
        .unwrap();
        let blinded_signature_c = PublicKey::from_hex(
            "02a9acc1e48c25eeeb9289b5031cc57da9fe72f3fe2861d264bdc074209b107ba2",
        )
        .unwrap();
        let e = "9818e061ee51d5c8edc3342369a554998ff7b4381c8652d724cdf46429be73d9";
        let s = "9818e061ee51d5c8edc3342369a554998ff7b4381c8652d724cdf46429be73da";

        let result = verify_dleq_proof_alice(
            e,
            s,
            &mint_pubkey_a,
            &blinded_message_b,
            &blinded_signature_c,
        );

        assert!(
            result.is_ok(),
            "Alice's DLEQ verification with NUT-12 test vector failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_nut12_carol_verification_vector() {
        // This test uses the vector for "DLEQ verification on Proof".
        // This is Carol's use case, where she verifies a token she received.
        let mint_pubkey_a = PublicKey::from_hex(
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        )
        .unwrap();
        let secret_str = "daf4dd00a2b68a0858a80450f52c8a7d2ccf87d375e43e216e0c571f089f63e9";
        let unblinded_signature_c = PublicKey::from_hex(
            "024369d2d22a80ecf78f3937da9d5f30c1b9f74f0c32684d583cca0fa6a61cdcfc",
        )
        .unwrap();

        let e = "b31e58ac6527f34975ffab13e70a48b6d2b0d35abc4b03f0151f09ee1a9763d4";
        let s = "8fbae004c59e754d71df67e392b6ae4e29293113ddc2ec86592a0431d16306d8";
        let r = "a6d13fcd7a18442e6076f5e1e7c887ad5de40a019824bdfa9fe740d302e8d861";

        // Carol must decode the secret's hex string to get the raw bytes for her calculations.
        let secret_bytes = hex::decode(secret_str).unwrap();

        let result = verify_dleq_proof_carol(
            e,
            s,
            r,
            &mint_pubkey_a,
            &secret_bytes,
            &unblinded_signature_c,
        );

        assert!(
            result.is_ok(),
            "Carol's DLEQ verification with NUT-12 test vector failed: {:?}",
            result.err()
        );
    }
}
