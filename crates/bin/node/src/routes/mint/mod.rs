mod outputs;

use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage},
    nut04::MintQuoteState,
};
use outputs::check_outputs_allow_single_unit;
use thiserror::Error;
use tonic::Status;
use tracing::{Level, event};
use uuid::Uuid;

use crate::{
    grpc_service::GrpcState,
    logic::{OutputsError, process_outputs},
    methods::Method,
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
            | Error::QuoteExpired => Status::deadline_exceeded(value.to_string()),
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

        let (expected_amount, state) =
            db_node::mint_quote::get_amount_and_state(&mut tx, quote).await?;

        if state != MintQuoteState::Paid {
            return Err(Error::InvalidQuoteStateAtThisPoint(state));
        }

        let (total_amount, _unit) =
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

        event!(
            name: "mint",
            Level::INFO,
            name = "mint",
            %method,
            quote_id = %quote,
        );

        let meter = opentelemetry::global::meter("business");
        let n_mint_counter = meter.u64_counter("mint.operation.count").build();
        n_mint_counter.add(1, &[]);

        Ok(blind_signatures)
    }
}
