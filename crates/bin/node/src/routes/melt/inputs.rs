use num_traits::CheckedAdd;
use std::collections::HashSet;
use std::sync::LazyLock;

use db_node::InsertSpentProofsQueryBuilder;
use nuts::{Amount, nut00::Proof};
use sqlx::PgConnection;

use crate::{
    app_state::SignerClient,
    keyset_cache::KeysetCache,
    logic::{InputsError, run_inputs_verification_queries},
};

// locally  defined felt and constants
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Felt([u8; 32]);

impl Felt {
    pub fn from_bytes_be(bytes: &[u8; 32]) -> Self {
        Felt(*bytes)
    }

    pub fn to_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn from_hex(hex: &str) -> Self {
        let mut bytes = [0u8; 32];
        let trimmed_hex = hex.trim_start_matches("0x");

        // Ensure the hex string has an even length
        if trimmed_hex.len() % 2 != 0 {
            panic!("Hex string has an odd length: {}", hex); // Improved error message
        }

        hex::decode_to_slice(trimmed_hex, &mut bytes).unwrap();
        Felt(bytes)
    }
}

impl std::fmt::Display for Felt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes()))
    }
}

pub static L2_ADDRESS_UPPER_BOUND: LazyLock<Felt> = LazyLock::new(|| {
    Felt::from_hex("0x8000000000000000000000000000000000000000000000000000000000000000")
});

pub static BLOCK_HASH_TABLE_ADDRESS: LazyLock<Felt> = LazyLock::new(|| {
    Felt::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001")
});

// Locally defined PublicKey struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey {
    pub key_data: [u8; 33], // Public keys are often 33 bytes in length
}

impl From<&nuts::nut01::PublicKey> for Felt {
    fn from(public_key: &nuts::nut01::PublicKey) -> Self {
        Felt::from_bytes_be(&public_key.to_bytes()[..32].try_into().unwrap())
    }
}
// Helper function to validate a Felt as a contract address
fn validate_address(y: &Felt) -> Result<(), crate::routes::melt::errors::Error> { 
    if *y > *BLOCK_HASH_TABLE_ADDRESS && *y < *L2_ADDRESS_UPPER_BOUND {
        Ok(())
    } else {
        Err(crate::routes::melt::errors::Error::InvalidAddress {
            addr: y.to_string(),
            message: format!(
                "Expected range: [0x2, {})",
                L2_ADDRESS_UPPER_BOUND.to_string()
            ),
        })
    }
}

pub async fn process_melt_inputs<'a>(
    conn: &mut PgConnection,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    inputs: &'a [Proof],
) -> Result<(Amount, InsertSpentProofsQueryBuilder<'a>), InputsError> {
    let mut common_unit = None;
    let mut secrets = HashSet::new();
    let mut query_builder = InsertSpentProofsQueryBuilder::new();
    let mut total_amount = Amount::ZERO;

    let mut verify_proofs_request = Vec::with_capacity(inputs.len());

    for proof in inputs {
        let y = proof.y().map_err(|_| InputsError::HashOnCurve)?;
        // Uniqueness
        if !secrets.insert(y) {
            Err(InputsError::DuplicateInput)?;
        }

        // convert y to felt before calling Validate address
        validate_address(&Felt::from(&y))?;

        let keyset_info = keyset_cache
            .get_keyset_info(conn, proof.keyset_id)
            .await
            .map_err(InputsError::KeysetCache)?;

        // Validate amount doesn't exceed max_order
        let max_order = keyset_info.max_order();
        let max_value = (1u64 << max_order) - 1;

        if u64::from(proof.amount) > max_value {
            return Err(InputsError::AmountExceedsMaxOrder(
                proof.keyset_id,
                proof.amount,
                max_value,
            ));
        }

        // Check all units are the same
        let unit = keyset_info.unit();
        match common_unit {
            Some(u) => {
                if u != unit {
                    Err(InputsError::MultipleUnits)?;
                }
            }
            None => common_unit = Some(unit),
        }

        // Incement total amount
        total_amount = total_amount
            .checked_add(&proof.amount)
            .ok_or(InputsError::TotalAmountTooBig)?;

        // Append to insert query
        query_builder.add_row(&y, proof);

        // Prepare payload for verification
        verify_proofs_request.push(signer::Proof {
            amount: proof.amount.into(),
            keyset_id: proof.keyset_id.to_bytes().to_vec(),
            secret: proof.secret.to_string(),
            unblind_signature: proof.c.to_bytes().to_vec(),
        });
    }

    run_inputs_verification_queries(conn, secrets, signer, verify_proofs_request).await?;

    Ok((total_amount, query_builder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::melt::errors::Error as MeltError;

    #[test]
    fn test_validate_address_valid() {
        // Address within the valid range
        let valid_address =
            Felt::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002");
        assert!(validate_address(&valid_address).is_ok());
    }

    #[test]
    fn test_validate_address_invalid_below_range() {
        // Address below the valid range
        let invalid_address =
            Felt::from_hex("0x0000000000000000000000000000000000000000000000000000000000000001");
        let result = validate_address(&invalid_address);
        assert!(result.is_err());
        if let Err(MeltError::InvalidAddress { addr, message }) = result {
            assert_eq!(addr, invalid_address.to_string());
            assert!(message.contains("Expected range"));
        }
    }

    #[test]
    fn test_validate_address_invalid_above_range() {
        // Address above the valid range
        let invalid_address = 
            Felt::from_hex("0x8000000000000000000000000000000000000000000000000000000000000001");
        let result = validate_address(&invalid_address);
        assert!(result.is_err());
        if let Err(MeltError::InvalidAddress { addr, message }) = result {
            assert_eq!(addr, invalid_address.to_string());
            assert!(message.contains("Expected range"));
        }
    }

    #[test]
    fn test_validate_address_edge_case_lower_bound() {
        // Address at the lower bound (just above BLOCK_HASH_TABLE_ADDRESS)
        let edge_case_address = 
            Felt::from_hex("0x0000000000000000000000000000000000000000000000000000000000000002");
        assert!(validate_address(&edge_case_address).is_ok());
    }

    #[test]
    fn test_validate_address_edge_case_upper_bound() {
        // Address at the upper bound (just below L2_ADDRESS_UPPER_BOUND)
        let edge_case_address = 
            Felt::from_hex("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        assert!(validate_address(&edge_case_address).is_ok());
    }
}