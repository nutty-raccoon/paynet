use crate::grpc_service::GrpcState;
use liquidity_source::DepositInterface;
use nuts::{
    Amount,
    nut04::{MintQuoteResponse, MintQuoteState},
};
use sqlx::PgConnection;
use starknet_types::Unit;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::{methods::Method, utils::unix_time};

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
    #[error("failed to serialize the quote request content")]
    SerQuoteRequest(serde_json::Error),
    // Mint quote specific errors
    #[error("Minting is currently disabled")]
    MintDisabled,
    #[error("Unsupported unit `{0}` for method `{1}`")]
    UnitNotSupported(Unit, Method),
    #[error("Amount must be at least {0}, got {1}")]
    AmountTooLow(Amount, Amount),
    #[error("Amount must bellow {0}, got {1}")]
    AmountTooHigh(Amount, Amount),
    #[error("failed to interact with liquidity source: {0}")]
    LiquiditySource(#[source] anyhow::Error),
}

impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match value {
            Error::TxBegin(error) | Error::TxCommit(error) | Error::Sqlx(error) => {
                Status::internal(error.to_string())
            }
            Error::Db(error) => Status::internal(error.to_string()),
            Error::SerQuoteRequest(error) => Status::internal(error.to_string()),
            Error::MintDisabled => Status::failed_precondition(value.to_string()),
            Error::UnitNotSupported(_, _)
            | Error::AmountTooLow(_, _)
            | Error::AmountTooHigh(_, _)
            | Error::LiquiditySource(_) => Status::invalid_argument(value.to_string()),
        }
    }
}

impl GrpcState {
    pub async fn inner_mint_quote(
        &self,
        method: Method,
        amount: Amount,
        unit: Unit,
    ) -> Result<MintQuoteResponse<Uuid>, Error> {
        // Release the lock asap
        let settings = {
            let read_nuts_settings_lock = self.nuts.read().await;

            if read_nuts_settings_lock.nut04.disabled {
                Err(Error::MintDisabled)?;
            }

            read_nuts_settings_lock
                .nut04
                .get_settings(method, unit)
                .ok_or(Error::UnitNotSupported(unit, method))?
        };

        if let Some(min_amount) = settings.min_amount {
            if min_amount > amount {
                Err(Error::AmountTooLow(min_amount, amount))?;
            }
        }
        if let Some(max_amount) = settings.max_amount {
            if max_amount < amount {
                Err(Error::AmountTooHigh(max_amount, amount))?;
            }
        }

        #[cfg(feature = "mock")]
        let depositer = liquidity_source::mock::MockDepositer;
        #[cfg(all(not(feature = "mock"), feature = "starknet"))]
        let depositer = self.starknet_config.depositer.clone();

        let mut conn = self.pg_pool.acquire().await?;
        let response = match method {
            Method::Starknet => create_new_starknet_mint_quote(
                &mut conn,
                depositer,
                amount,
                unit,
                self.quote_ttl.mint_ttl(),
            ),
        }
        .await?;

        Ok(response)
    }
}

/// Initialize a new Starknet mint quote
async fn create_new_starknet_mint_quote(
    conn: &mut PgConnection,
    depositer: impl DepositInterface,
    amount: Amount,
    unit: Unit,
    mint_ttl: u64,
) -> Result<MintQuoteResponse<Uuid>, Error> {
    let expiry = unix_time() + mint_ttl;
    let quote = Uuid::new_v4();
    let quote_hash = bitcoin_hashes::Sha256::hash(quote.as_bytes());

    let request = depositer
        .generate_deposit_payload(quote_hash, unit, amount)
        .map_err(|e| Error::LiquiditySource(e.into()))?;

    db_node::mint_quote::insert_new(
        conn,
        quote,
        quote_hash.as_byte_array(),
        unit,
        amount,
        &request,
        expiry,
    )
    .await
    .map_err(Error::Db)?;

    let state = {
        // If running with no backend, we immediatly set the state to paid
        #[cfg(feature = "mock")]
        {
            use futures::TryFutureExt;

            let new_state = MintQuoteState::Paid;
            db_node::mint_quote::set_state(conn, quote, new_state)
                .map_err(Error::Sqlx)
                .await?;
            new_state
        }
        #[cfg(all(not(feature = "mock"), feature = "starknet"))]
        MintQuoteState::Unpaid
    };

    Ok(MintQuoteResponse {
        quote,
        request,
        state,
        expiry,
    })
}
