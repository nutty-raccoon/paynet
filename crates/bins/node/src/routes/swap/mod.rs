mod inputs;

use inputs::process_swap_inputs;
use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage, Proof},
};
use starknet_types::Unit;
use thiserror::Error;
use tonic::Status;
use tonic_types::{ErrorDetails, StatusExt};
use tracing::{Level, event};

use crate::{
    app_state::AppState,
    logic::{InputsError, OutputsError, check_outputs_allow_multiple_units, process_outputs},
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
    // Primitive processing errors
    #[error(transparent)]
    Outputs(#[from] OutputsError),
    #[error(transparent)]
    Inputs(#[from] InputsError),
    // Swap specific errors
    #[error("All input units should be present as output")]
    UnbalancedUnits,
    #[error("For unit {0}, Inputs: `{1}`, Outputs: `{2}`")]
    TransactionUnbalanced(Unit, Amount, Amount),
    #[error("the sum off all the outputs' amount and the fee must fit in a u64")]
    TotalOutputAndFeeTooBig,
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::TxBegin(error) | Error::TxCommit(error) | Error::Sqlx(error) => {
                Status::internal(error.to_string())
            }
            Error::Outputs(error) => match error {
                OutputsError::DuplicateOutput
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
                OutputsError::InactiveKeyset(keyset_id) => Status::with_error_details(
                    tonic::Code::FailedPrecondition,
                    "inactive keyset",
                    ErrorDetails::with_precondition_failure_violation(
                        "keyset.state",
                        format!("keyset/{}", keyset_id),
                        "Keyset must be active to perform this operation",
                    ),
                ),
            },
            Error::Inputs(error) => error.into(),
            Error::UnbalancedUnits
            | Error::TransactionUnbalanced(_, _, _)
            | Error::TotalOutputAndFeeTooBig => Status::invalid_argument(value.to_string()),
        }
    }
}

impl AppState {
    pub async fn inner_swap(
        &self,
        inputs: &[Proof],
        outputs: &[BlindedMessage],
    ) -> Result<Vec<BlindSignature>, Error> {
        let mut tx = db_node::begin_db_tx(&self.pg_pool)
            .await
            .map_err(Error::TxBegin)?;

        let outputs_amounts =
            check_outputs_allow_multiple_units(&mut tx, self.keyset_cache.clone(), outputs)
                .await
                .map_err(Error::Outputs)?;

        let (input_fees_and_amount, insert_spent_proofs_query_builder) = process_swap_inputs(
            &mut tx,
            self.signer.clone(),
            self.keyset_cache.clone(),
            inputs,
        )
        .await
        .map_err(Error::Inputs)?;

        // Amount matching
        for (unit, output_amount) in outputs_amounts.iter() {
            let &(_, input_amount) = input_fees_and_amount
                .iter()
                .find(|(u, _)| u == unit)
                .ok_or(Error::UnbalancedUnits)?;

            if input_amount != *output_amount {
                Err(Error::TransactionUnbalanced(
                    *unit,
                    input_amount,
                    *output_amount,
                ))?;
            }
        }

        // Output process
        let (blind_signatures, insert_blind_signatures_query_builder) =
            process_outputs(self.signer.clone(), outputs).await?;

        insert_spent_proofs_query_builder.execute(&mut tx).await?;
        insert_blind_signatures_query_builder
            .execute(&mut tx)
            .await?;

        tx.commit().await.map_err(Error::TxCommit)?;

        event!(
            name: "swap",
            Level::INFO,
            amounts = serde_json::to_string(&outputs_amounts).unwrap(),
            "Swap"
        );
        let meter = opentelemetry::global::meter("business");
        let n_swap_counter = meter.u64_counter("swap.operation.count").build();
        n_swap_counter.add(1, &[]);
        for (u, a) in outputs_amounts {
            let amount_of_unit_swaped_counter = meter
                .u64_counter(format!("swap.amount.{}.count", &u))
                .with_unit(u.as_str())
                .build();
            amount_of_unit_swaped_counter.add(a.into(), &[]);
        }

        Ok(blind_signatures)
    }
}
