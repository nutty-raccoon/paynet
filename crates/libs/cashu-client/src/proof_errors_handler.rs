use crate::{Error, GrpcClient};
use tonic::Code;
use tonic_types::StatusExt;

#[derive(Debug)]
pub enum ProofErrorKind {
    AlreadySpent,
    FailCryptoVerify,
    Unknown(String),
}

#[derive(Debug)]
pub struct ProofError {
    pub index: Vec<u32>,
    pub kind: ProofErrorKind,
}

pub trait ProofErrorHandler {
    fn extract_proof_errors(
        &self,
        raw: &(dyn std::error::Error + 'static),
    ) -> Option<Vec<ProofError>>;
    fn keyset_refresh(&self, raw: &(dyn std::error::Error + 'static)) -> bool;
}

impl ProofErrorHandler for GrpcClient {
    fn extract_proof_errors(
        &self,
        raw: &(dyn std::error::Error + 'static),
    ) -> Option<Vec<ProofError>> {
        if let Some(status) = raw.downcast_ref::<tonic::Status>() {
            if let Some(bad_request) = status.get_error_details().bad_request() {
                let mut spent = Vec::new();
                let mut invalid = Vec::new();
                for violation in &bad_request.field_violations {
                    let idx = extract_proof_index(&violation.field).unwrap_or(0);
                    if violation.description.contains("already spent") {
                        spent.push(idx);
                    } else if violation
                        .description
                        .contains("failed cryptographic verification")
                    {
                        invalid.push(idx);
                    }
                }
                let errs = vec![
                    ProofError {
                        index: spent,
                        kind: ProofErrorKind::AlreadySpent,
                    },
                    ProofError {
                        index: invalid,
                        kind: ProofErrorKind::FailCryptoVerify,
                    },
                ];
                return Some(errs);
            }
        }
        None
    }

    fn keyset_refresh(&self, raw: &(dyn std::error::Error + 'static)) -> bool {
        if let Some(status) = raw.downcast_ref::<tonic::Status>() {
            if status.code() == Code::FailedPrecondition && status.message() == "inactive keyset" {
                let error_details = status.get_error_details();
                if let Some(precondition_failure) = error_details.precondition_failure() {
                    for failure in &precondition_failure.violations {
                        if failure.r#type == "keyset.state" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

fn extract_proof_index(field: &str) -> Result<u32, Error> {
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
