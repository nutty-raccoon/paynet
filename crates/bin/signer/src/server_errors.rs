use nuts::{Amount, dhke, nut02::KeysetId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error(transparent)]
    Dhke(#[from] dhke::Error),
    #[error("The lock has been poisoined")]
    LockPoisoned,
    #[error("Keyset with id {0} not found")]
    KeysetNotFound(KeysetId),
    #[error("Amount {0} not found in keyset with id {1}")]
    AmountNotFound(Amount, KeysetId),
    #[error("Unkown method {0}")]
    UnknownMethod(&'a str),
    #[error("Unkown unit {0}")]
    UnknownUnit(&'a str),
    #[error("max_order should be no greater than u8::MAX")]
    MaxOrderTooBig,
    #[error("Invalid keyset id")]
    BadKeysetId,
    #[error("Invalid secret")]
    BadSecret,
    #[error("Invalid secret")]
    BadC,
}

impl<'a> From<Error<'a>> for String {
    fn from(val: Error<'a>) -> Self {
        val.to_string()
    }
}
