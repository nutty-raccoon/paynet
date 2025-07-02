use nuts::Amount;
use starknet_types::Unit;
use tonic::Status;

use crate::{logic::InputsError, methods::Method};

use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to commit db tx: {0}")]
    TxCommit(#[source] sqlx::Error),
    #[error("failed to begin db tx: {0}")]
    TxBegin(#[source] sqlx::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Db(#[from] db_node::Error),
    #[error("melting is disabled")]
    MeltDisabled,
    #[error("unsupported unit `{0}` for method `{1}`")]
    UnitNotSupported(Unit, Method),
    #[error("could not convert asset amount to unit")]
    InvalidAssetConversion,
    #[error("total input amount {0} does not match required amount {1}")]
    InvalidAmount(Amount, Amount),
    #[error("melt quote `{0}` not found")]
    QuoteNotFound(Uuid),
    #[error("melt quote `{0}` is expired")]
    QuoteExpired(Uuid),
    #[error("melt quote `{0}` has already been processed")]
    QuoteAlreadyProcessed(Uuid),
    #[error("the sum of all the inputs' amount must fit in a u64")]
    TotalAmountTooBig,
    #[error(transparent)]
    Inputs(#[from] InputsError),
    #[error("total input amount {0} is lower than the minimum required {1}")]
    AmountTooLow(Amount, Amount),
    #[error("total input amount {0} is higher than the maximum allowed {1}")]
    AmountTooHigh(Amount, Amount),
    #[error(transparent)]
    InvalidPaymentRequest(serde_json::Error),
    #[error("failed to interact with liquidity source: {0}")]
    LiquiditySource(#[source] anyhow::Error),
    #[error("method '{0}' not supported, try compiling with the appropriate feature.")]
    MethodNotSupported(Method),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::TxBegin(error) | Error::TxCommit(error) | Error::Sqlx(error) => {
                Status::internal(error.to_string())
            }
            Error::UnitNotSupported(_, _)
            | Error::AmountTooLow(_, _)
            | Error::AmountTooHigh(_, _)
            | Error::TotalAmountTooBig
            | Error::MethodNotSupported(_)
            | Error::InvalidPaymentRequest(_) => Status::invalid_argument(value.to_string()),
            Error::Inputs(error) => error.into(),
            Error::Db(error) => Status::internal(error.to_string()),
            Error::MeltDisabled => Status::failed_precondition(value.to_string()),
            Error::LiquiditySource(_) => Status::internal(value.to_string()),
            Error::QuoteNotFound(_) | Error::QuoteExpired(_) | Error::QuoteAlreadyProcessed(_) => {
                Status::not_found(value.to_string())
            }
            Error::InvalidAssetConversion => Status::failed_precondition(value.to_string()),
            Error::InvalidAmount(_, _) => Status::failed_precondition(value.to_string()),
        }
    }
}
