use nuts::{
    Amount,
    nut00::{BlindSignature, BlindedMessage, Proof},
};
use starknet_types::Unit;
use thiserror::Error;
use tonic::Status;

use crate::{
    grpc_service::GrpcState,
    logic::{
        InputsError, OutputsError, check_outputs_allow_multiple_units, process_outputs,
        process_swap_inputs,
    },
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
            Error::Inputs(error) => error.into(),
            Error::UnbalancedUnits
            | Error::TransactionUnbalanced(_, _, _)
            | Error::TotalOutputAndFeeTooBig => Status::invalid_argument(value.to_string()),
        }
    }
}

impl GrpcState {
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
        for (asset, output_amount) in outputs_amounts {
            let &(_, input_amount) = input_fees_and_amount
                .iter()
                .find(|(u, _)| *u == asset)
                .ok_or(Error::UnbalancedUnits)?;

            if input_amount != output_amount {
                Err(Error::TransactionUnbalanced(
                    asset,
                    input_amount,
                    output_amount,
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

        Ok(blind_signatures)
    }
}
