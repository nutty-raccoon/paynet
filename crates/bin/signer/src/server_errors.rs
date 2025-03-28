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
    AmountGreaterThanMax(usize, Amount, Amount),
    AmountNotPowerOfTwo(usize, Amount),
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
            Error::AmountGreaterThanMax(idx, amount, max_order) => Status::with_error_details(
                Code::InvalidArgument,
                "amount is greater than max order",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("messages[{idx}].amount"),
                    format!(
                        "the provided amount {amount} is greater than the max order: {max_order}"
                    ),
                )]),
            ),
            Error::AmountNotPowerOfTwo(idx, amount) => Status::with_error_details(
                Code::InvalidArgument,
                "amount is not a power of two",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("messages[{idx}].amount"),
                    format!("the provided amount {amount} is not a power of two"),
                )]),
            ),
            Error::CouldNotSignMessage(idx, message, error) => Status::with_error_details(
                Code::InvalidArgument,
                "failed to sign message",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("messages[{}].blinded_secret", idx),
                    format!(
                        "given message {message} the resulting key would have been invalid: {error}"
                    ),
                )]),
            ),
            Error::CouldNotVerifyProof(idx, proof, secret, error) => Status::with_error_details(
                Code::InvalidArgument,
                "failed to verify proof",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("proofs[{}]", idx),
                    format!(
                        "given proof {proof}, secret {secret} and the service private key the resulting key would have been invalid: {error}",
                    ),
                )]),
            ),
            Error::BadKeysetId(field, idx, bad_keyset_id, error) => Status::with_error_details(
                Code::InvalidArgument,
                "invalid keyset id",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{}[{}].keyset_id", field, idx),
                    format!(
                        "the provided keyset id '{:?}' is invalid: {error}",
                        bad_keyset_id,
                    ),
                )]),
            ),
            Error::KeysetNotFound(field, idx, keyset_id) => Status::with_error_details(
                Code::NotFound,
                "keyset not found",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{field}[{idx}].keyset_id"),
                    format!("the specified keyset id '{keyset_id}' does not exist"),
                )]),
            ),
            Error::AmountNotFound(field, idx, keyset_id, amount) => Status::with_error_details(
                Code::NotFound,
                "amount not found",
                ErrorDetails::with_bad_request(vec![FieldViolation::new(
                    format!("{field}[{idx}].amount"),
                    format!("amount {amount} does not exist in the keyset with id {keyset_id}"),
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
                        Unit::MilliStrk
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
