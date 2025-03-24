use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut04::MintQuoteState,
};
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::{
    grpc_service::GrpcState,
    logic::{OutputsError, check_outputs_allow_single_unit, process_outputs},
    methods::Method,
    utils::unix_time,
};

#[derive(Debug, Error)]
pub enum Error {
    // Db errors
    #[error("failed to commit db tx: {0}")]
    TxCommit(#[source] sqlx::Error),
    #[error("failed to commit db tx: {0}")]
    TxBegin(#[source] sqlx::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Db(#[from] db_node::Error),
    #[error(transparent)]
    Outputs(#[from] OutputsError),
    #[error("Invalid quote state {0} at this poin of the flow")]
    InvalidQuoteStateAtThisPoint(MintQuoteState),
    #[error(
        "The outputs' total amount {expected} doesn't match the one specified in the quote {received}"
    )]
    OutputsAmount { expected: Amount, received: Amount },
    #[error("Quote has expired")]
    QuoteExpired,
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::TxBegin(error) | Error::TxCommit(error) | Error::Sqlx(error) => {
                Status::internal(error.to_string())
            }
            Error::Db(error) => Status::internal(error.to_string()),
            Error::Outputs(error) => match error {
                OutputsError::DuplicateOutput
                | OutputsError::InactiveKeyset(_)
                | OutputsError::MultipleUnits
                | OutputsError::TotalAmountTooBig
                | OutputsError::AlreadySigned
                | OutputsError::AmountExceedsMaxOrder(_, _, _) => {
                    Status::invalid_argument(error.to_string())
                }
                OutputsError::Db(sqlx::Error::RowNotFound) => Status::not_found(error.to_string()),
                OutputsError::Db(_) | OutputsError::KeysetCache(_) => {
                    Status::internal(error.to_string())
                }
                OutputsError::Signer(status) => status,
            },
            Error::InvalidQuoteStateAtThisPoint(_)
            | Error::OutputsAmount { .. }
            | Error::QuoteExpired => Status::invalid_argument(value.to_string()),
        }
    }
}

impl GrpcState {
    pub async fn inner_mint(
        &self,
        method: Method,
        quote: Uuid,
        outputs: &[BlindedMessage],
    ) -> Result<Vec<BlindSignature>, Error> {
        match method {
            Method::Starknet => {}
        }

        let mut tx = db_node::begin_db_tx(&self.pg_pool).await?;

        let quote_response = db_node::mint_quote::build_response_from_db(&mut tx, quote).await?;

        let (expected_amount, state) =
            db_node::mint_quote::get_amount_and_state(&mut tx, quote).await?;

        let current_time = unix_time();

        if current_time > quote_response.expiry {
            return Err(Error::QuoteExpired);
        }

        if state != MintQuoteState::Paid {
            return Err(Error::InvalidQuoteStateAtThisPoint(state));
        }

        let total_amount =
            check_outputs_allow_single_unit(&mut tx, &self.keyset_cache, outputs).await?;

        if total_amount != expected_amount {
            return Err(Error::OutputsAmount {
                expected: expected_amount,
                received: total_amount,
            });
        }

        let (blind_signatures, insert_blind_signatures_query_builder) =
            process_outputs(self.signer.clone(), outputs).await?;

        insert_blind_signatures_query_builder
            .execute(&mut tx)
            .await?;
        db_node::mint_quote::set_state(&mut tx, quote, MintQuoteState::Issued).await?;

        tx.commit().await?;

        Ok(blind_signatures)
    }
}
