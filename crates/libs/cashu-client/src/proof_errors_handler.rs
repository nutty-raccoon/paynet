use crate::grpc_client::Error;

#[derive(Debug)]
pub enum ProofErrorKind {
    AlreadySpent,
    FailCryptoVerify,
}

#[derive(Debug)]
pub struct ProofError {
    pub indexes: Vec<u32>,
    pub kind: ProofErrorKind,
}

pub fn extract_proof_index(field: &str) -> Result<u32, Error> {
    if let Some(start) = field.find('[') {
        if let Some(end) = field.find(']') {
            let index_str = &field[start + 1..end];
            return Ok(index_str.parse::<u32>()?);
        }
    }

    Err(Error::InvalidFormat)
}

#[cfg(test)]
mod tests {
    use super::extract_proof_index;

    #[test]
    fn test_extract_proof_index_valid_input() {
        let cases = [
            ("proofs[5]", 5),
            ("proofs[10]", 10),
            ("proofs[123]", 123),
            ("proofs[99]", 99),
        ];

        cases.iter().for_each(|(field, expected)| {
            assert_eq!(extract_proof_index(field).unwrap(), *expected)
        });
    }

    #[test]
    fn test_extract_proof_index_invalid_format() {
        let cases = ["proofs5]", "proofs[5", "", "proofs74]", "proofs[980"];

        cases.iter().for_each(|field| {
            assert!(extract_proof_index(field).is_err());
            assert_eq!(
                extract_proof_index(field).unwrap_err().to_string(),
                "invalid field format: '[' or ']' not found"
            )
        });
    }

    #[test]
    fn test_extract_proof_index_parse_error() {
        let cases = ["proofs[abc]", "proofs[1.2]", "proofs[9a]", "proofs[a4]"];

        cases.iter().for_each(|field| {
            assert!(extract_proof_index(field).is_err());
            assert_eq!(
                extract_proof_index(field).unwrap_err().to_string(),
                "invalid index: invalid digit found in string"
            );
        });
    }

    #[test]
    fn test_extract_proof_index_edge_cases() {
        let cases: [(&str, u32); 2] = [("proofs[0]", 0), ("proofs[4294967295]", 4294967295)];

        cases.iter().for_each(|(field, element)| {
            assert_eq!(extract_proof_index(field).unwrap(), *element);
        });

        assert!(extract_proof_index("proofs[4294967296]").is_err());
        assert!(extract_proof_index("proofs[-1]").is_err());
    }
}
