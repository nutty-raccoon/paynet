use crate::grpc_service::GrpcState;
use nuts::{
    Amount,
    nut04::{MintQuoteResponse, MintQuoteState},
};
use sqlx::PgPool;
use starknet_types::{MintPaymentRequest, PayInvoiceCalldata};
use starknet_types_core::felt::Felt;
use thiserror::Error;
use tonic::Status;
use uuid::Uuid;

use crate::{Unit, methods::Method, utils::unix_time};

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
            | Error::AmountTooHigh(_, _) => Status::invalid_argument(value.to_string()),
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

        let response = match method {
            Method::Starknet => {
                new_starknet_mint_quote(&self.pg_pool, amount, unit, self.quote_ttl.mint_ttl())
            }
        }
        .await?;

        Ok(response)
    }
}

/// Initialize a new Starknet mint quote
async fn new_starknet_mint_quote(
    pool: &PgPool,
    amount: Amount,
    unit: Unit,
    mint_ttl: u64,
) -> Result<MintQuoteResponse<Uuid>, Error> {
    let expiry = unix_time() + mint_ttl;
    let quote = Uuid::new_v4();
    let quote_hash = bitcoin_hashes::Sha256::hash(quote.as_bytes());

    let request = {
        let asset = unit.asset();
        let amount = unit.convert_amount_into_u256(amount);

        serde_json::to_string(&MintPaymentRequest {
            contract_address: Felt::from_hex_unchecked(
                "0x03a94f47433e77630f288054330fb41377ffcc49dacf56568eeba84b017aa633",
            ),
            selector: Felt::from_hex_unchecked(
                "0x027a12f554d018764f982295090da45b4ff0734785be0982b62c329b9ac38033",
            ),
            calldata: PayInvoiceCalldata {
                invoice_id: quote.as_u128(),
                asset,
                amount,
            },
        })
        .map_err(Error::SerQuoteRequest)?
    };

    let mut conn = pool.acquire().await?;
    db_node::mint_quote::insert_new(
        &mut conn,
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
        #[cfg(not(feature = "starknet"))]
        {
            use futures::TryFutureExt;

            let new_state = MintQuoteState::Paid;
            db_node::mint_quote::set_state(&mut conn, quote, new_state)
                .map_err(Error::Sqlx)
                .await?;
            new_state
        }
        #[cfg(feature = "starknet")]
        MintQuoteState::Unpaid
    };

    Ok(MintQuoteResponse {
        quote,
        request,
        state,
        expiry,
    })
}
