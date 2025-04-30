use nuts::Amount;
use starknet_types::{Asset, Unit};
// use tonic::Status;

use crate::{logic::InputsError, methods::Method};

#[derive(Debug, thiserror::Error)]
pub enum Error{
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
    #[error("unsupported unit `{0}` for method `{1}`")]
    UnitNotSupported(Unit, Method),
    #[error("unsupported asset `{0}` for unit `{1}`")]
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
    #[error("failed to interact with liquidity source: {0}")]
    LiquiditySource(#[source] anyhow::Error),
    #[error("method '{0}' not supported, try compiling with the appropriate feature.")]
    MethodNotSupported(Method),
    #[error("invalid address `{addr}`: {message}")]
    InvalidAddress { addr: String, message: String }, // Added new error variant for invalid address
    #[error("other error: {0}")]
    Other(String),
}

impl From<Error> for tonic::Status {
    fn from(error: Error) -> Self {
        match error {
            Error::InvalidAddress { addr, message } => {
                tonic::Status::invalid_argument(format!("Invalid address `{addr}`: {message}"))
            }
            _ => tonic::Status::internal(error.to_string()),
        }
    }
}


