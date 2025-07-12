use crate::grpc_service::GrpcState;
use liquidity_source::{DepositInterface, LiquiditySource};
use nuts::{
    Amount,
    nut04::{MintQuoteResponse, MintQuoteState},
};
use sqlx::PgConnection;
use starknet_types::Unit;
use thiserror::Error;
use tonic::Status;
use tracing::{Level, event};
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
    #[error("method '{0}' not supported, try compiling with the appropriate feature.")]
    MethodNotSupported(Method),
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
            | Error::MethodNotSupported(_)
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

        let liquidity_source = self
            .liquidity_sources
            .get_liquidity_source(method)
            .ok_or(Error::MethodNotSupported(method))?;

        let mut conn = self.pg_pool.acquire().await?;
        let response = match method {
            Method::Starknet => create_new_mint_quote(
                &mut conn,
                liquidity_source.depositer(),
                amount,
                unit,
                self.quote_ttl.mint_ttl(),
            ),
        }
        .await?;

        event!(
            name: "mint-quote",
            Level::INFO,
            name = "mint-quote",
            %method,
            amount = u64::from(amount),
            %unit,
            quote_id = %response.quote,
        );

        Ok(response)
    }
}

/// Initialize a new mint quote
async fn create_new_mint_quote(
    conn: &mut PgConnection,
    depositer: impl DepositInterface,
    amount: Amount,
    unit: Unit,
    mint_ttl: u64,
) -> Result<MintQuoteResponse<Uuid>, Error> {
    let expiry = unix_time() + mint_ttl;
    let quote_id = Uuid::new_v4();

    let (invoice_id, request) = depositer
        .generate_deposit_payload(quote_id, unit, amount, expiry)
        .map_err(|e| Error::LiquiditySource(e.into()))?;

    db_node::mint_quote::insert_new(
        conn,
        quote_id,
        invoice_id.into(),
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
            db_node::mint_quote::set_state(conn, quote_id, new_state)
                .map_err(Error::Sqlx)
                .await?;
            new_state
        }

        #[cfg(all(not(feature = "mock"), feature = "starknet"))]
        MintQuoteState::Unpaid
    };

    Ok(MintQuoteResponse {
        quote: quote_id,
        request,
        state,
        expiry,
    })
}
