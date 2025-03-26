use nuts::Amount;
use starknet_types::{Asset, Unit};
use tonic::Status;

use crate::{logic::InputsError, methods::Method};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to commit db tx: {0}")]
    TxCommit(#[source] sqlx::Error),
    #[error("failed to commit db tx: {0}")]
    TxBegin(#[source] sqlx::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Db(#[from] db_node::Error),
    #[error("failed to serialize the request content")]
    MeltDisabled,
    #[error("Unsupported unit `{0}` for method `{1}`")]
    UnitNotSupported(Unit, Method),
    #[error("Unsupported asset `{0}` for unit `{1}`")]
    InvalidAssetForUnit(Asset, Unit),
    #[error("the sum off all the inputs' amount must fit in a u64")]
    TotalAmountTooBig,
    #[error(transparent)]
    Inputs(#[from] InputsError),
    #[error("total inputs's amount {0} is lower than the node minimal amount {1} ")]
    AmountTooLow(Amount, Amount),
    #[error("total inputs's amount {0} is higher than the node maximal amount {1} ")]
    AmountTooHigh(Amount, Amount),
    #[error(transparent)]
    InvalidPaymentRequest(serde_json::Error),
    #[cfg(feature = "starknet")]
    #[error("failed to trigger withdraw from starknet cashier: {0}")]
    StarknetCashier(#[source] tonic::Status),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::TxBegin(error) | Error::TxCommit(error) | Error::Sqlx(error) => {
                Status::internal(error.to_string())
            }
            Error::UnitNotSupported(_, _)
            | Error::InvalidAssetForUnit(_, _)
            | Error::AmountTooLow(_, _)
            | Error::AmountTooHigh(_, _)
            | Error::TotalAmountTooBig
            | Error::InvalidPaymentRequest(_) => Status::invalid_argument(value.to_string()),
            Error::Inputs(error) => error.into(),
            Error::Db(error) => Status::internal(error.to_string()),
            Error::MeltDisabled => Status::failed_precondition(value.to_string()),
            #[cfg(feature = "starknet")]
            Error::StarknetCashier(_) => Status::internal(value.to_string()),
        }
    }
}
