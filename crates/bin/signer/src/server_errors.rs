use nuts::{
    Amount, dhke,
    nut01::{self, PublicKey},
    nut02::{self, KeysetId},
};
use starknet_types::Unit;
use tonic::{Code, Status};
use tonic_types::{ErrorDetails, FieldViolation, StatusExt};

#[derive(Debug)]
pub enum Error<'a> {
    UnknownUnit(&'a str),
    MaxOrderTooBig(u32),
    CouldNotSignMessage(usize, PublicKey, dhke::Error),
    CouldNotVerifyProof(usize, PublicKey, String, dhke::Error),
    BadKeysetId(&'a str, usize, &'a [u8], nut02::Error),
    KeysetNotFound(&'a str, usize, KeysetId),
    AmountNotFound(&'a str, usize, KeysetId, Amount),
    BadSecret(usize, nut01::Error),
    InvalidSignature(usize, nut01::Error),
}

impl<'a> From<Error<'a>> for Status {
    fn from(err: Error<'a>) -> Self {
        match err {
            Error::CouldNotSignMessage(idx, message, error) => Status::with_error_details(
                Code::InvalidArgument,
                format!("failed to sign message {}: {}", message, error),
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("messages[{}].blinded_secret", idx),
                    "the resulting key would have been invalid",
                )]),
            ),
            Error::CouldNotVerifyProof(idx, proof, secret, error) => Status::with_error_details(
                Code::InvalidArgument,
                format!(
                    "failed to verify proof {} on secret {}: {}",
                    proof, secret, error
                ),
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("proofs[{}]", idx),
                    "the resulting key would have been invalid",
                )]),
            ),
            Error::BadKeysetId(field, idx, bad_keyset_id, error) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid keyset id",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{}[{}].keyset_id", field, idx),
                    format!(
                        "the provided keyset id '{:?}' is invalid: {}",
                        bad_keyset_id, error
                    ),
                )]),
            ),
            Error::KeysetNotFound(field, idx, keyset_id) => Status::with_error_details(
                Code::NotFound,
                format!("keyset with id {} not found", keyset_id),
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{field}[{idx}].keyset_id"),
                    "the specified keyset id does not exist",
                )]),
            ),
            Error::AmountNotFound(field, idx, keyset_id, amount) => Status::with_error_details(
                Code::NotFound,
                format!(
                    "amount {} not found in keyset with id {}",
                    amount, keyset_id
                ),
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{field}[{idx}].amount"),
                    "the specified amount does not exist in the keyset",
                )]),
            ),
            Error::UnknownUnit(unit) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid unit",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    "unit",
                    format!(
                        "{} is not part of the units currently supported: [{}]",
                        unit,
                        Unit::Strk
                    ),
                )]),
            ),
            Error::MaxOrderTooBig(max_order) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid max_order",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    "max_order",
                    format!(
                        "the provided value {} should not exceeds u8::MAX ({})",
                        max_order,
                        u8::MAX
                    ),
                )]),
            ),
            Error::BadSecret(idx, error) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid secret",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("messages[{idx}].secret"),
                    format!("the provided secret is invalid: {error}"),
                )]),
            ),
            Error::InvalidSignature(idx, error) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid signature",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("proofs[{}].unblind_signature", idx),
                    format!("the provided signature is invalid: {}", error),
                )]),
            ),
        }
    }
}
