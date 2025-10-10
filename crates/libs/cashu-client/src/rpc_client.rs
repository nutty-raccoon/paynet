use nuts::{
    nut00::{BlindSignature, BlindedMessage},
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
    nut03::{SwapRequest, SwapResponse},
    nut04::{self, MintQuoteResponse, MintQuoteState, MintRequest},
    nut05::{MeltQuoteState, MeltRequest, MeltResponse},
    nut07::{CheckStateResponse, ProofCheckState},
    Amount,
};

use crate::{
    CashuClient, CashuClientError, ClientKey, ClientKeysResponse, ClientKeyset, ClientKeysetKeys,
    ClientKeysetsResponse, ClientMeltQuoteRequest, ClientMeltQuoteResponse, ClientMintQuoteRequest,
    ClientRestoreResponse, Error, NodeInfoResponse,
    proof_errors_handler::{ProofError, ProofErrorKind, extract_proof_index},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("RPC error: {0}")]
    Rpc(#[from] ),
    #[error("Invalid State in {method}")]
    InvalidState { method: String },
    #[error(transparent)]
    KeysetId(nut02::Error),
    #[error(transparent)]
    PublicKey(nut01::Error),
    #[error(transparent)]
    Method(nut04::Error),
    #[error("invalid field format: '[' or ']' not found")]
    InvalidFormat,
    #[error("invalid index: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
}

impl From<Error> for CashuClientError {
    fn from(value: Error) -> Self {
        match value {
            Error::Grpc(status) => {
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
                            indexes: spent,
                            kind: ProofErrorKind::AlreadySpent,
                        },
                        ProofError {
                            indexes: invalid,
                            kind: ProofErrorKind::FailCryptoVerify,
                        },
                    ];

                    return CashuClientError::Proof(errs);
                } else if status.message() == "inactive keyset"
                    && status.code() == tonic::Code::FailedPrecondition
                {
                    let error_details = status.get_error_details();
                    if let Some(precondition_failure) = error_details.precondition_failure() {
                        for failure in &precondition_failure.violations {
                            if failure.r#type == "keyset.state" {
                                return CashuClientError::InactiveKeyset;
                            }
                        }
                    }
                }

                CashuClientError::Other(Box::new(status))
            }

            e => CashuClientError::Other(Box::new(e)),
        }
    }
}