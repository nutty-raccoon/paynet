mod errors;
mod inputs;

use std::time::Duration;

use inputs::process_melt_inputs;
use liquidity_source::{LiquiditySource, WithdrawInterface};
use nuts::Amount;
use nuts::nut00::Proof;
use nuts::nut05::MeltQuoteState;
use starknet_types::Unit;
use tracing::{Level, event};
use uuid::Uuid;

use crate::utils::unix_time;
use crate::{grpc_service::GrpcState, methods::Method};

use errors::Error;

impl GrpcState {
    /// Step 1: Create a melt quote (NUT-05)
    /// This only validates the payment request and creates a quote - no payment processing
    pub async fn inner_melt_quote(
        &self,
        method: Method,
        unit: Unit,
        melt_payment_request: String,
    ) -> Result<nuts::nut05::MeltQuoteResponse<Uuid, Unit>, Error> {
        // Release the lock asap
        let settings = {
            let read_nuts_settings_lock = self.nuts.read().await;

            if read_nuts_settings_lock.nut05.disabled {
                Err(Error::MeltDisabled)?;
            }

            read_nuts_settings_lock
                .nut05
                .get_settings(method, unit)
                .ok_or(Error::UnitNotSupported(unit, method))?
        };

        let liquidity_source = self
            .liquidity_sources
            .get_liquidity_source(method)
            .ok_or(Error::MethodNotSupported(method))?;
        let withdrawer = liquidity_source.withdrawer();

        // Validate the payment request format
        let payment_request = withdrawer
            .deserialize_payment_request(&melt_payment_request)
            .map_err(|e| Error::LiquiditySource(e.into()))?;

        // No fee for now
        let fee = Amount::ZERO;
        let total_amount = withdrawer
            .compute_total_amount_expected(payment_request, unit, fee)
            .map_err(|e| Error::LiquiditySource(e.into()))?;

        let expiry = unix_time() + self.quote_ttl.melt_ttl();
        let quote_id = Uuid::new_v4();
        let quote_hash = bitcoin_hashes::Sha256::hash(quote_id.as_bytes());

        // Store the quote in database
        let mut conn = self.pg_pool.acquire().await?;
        db_node::melt_quote::insert_new(
            &mut conn,
            quote_id,
            quote_hash.as_byte_array(),
            settings.unit,
            total_amount,
            fee,
            &melt_payment_request,
            expiry,
        )
        .await?;

        Ok(nuts::nut05::MeltQuoteResponse {
            quote: quote_id,
            unit,
            amount: total_amount,
            state: nuts::nut05::MeltQuoteState::Unpaid,
            expiry,
        })
    }

    /// Step 2: Execute the melt using an existing quote ID
    /// This processes the actual payment using the previously created quote
    pub async fn inner_melt(
        &self,
        method: Method,
        quote_id: Uuid,
        inputs: &[Proof],
    ) -> Result<Option<Vec<String>>, Error> {
        let mut conn = self.pg_pool.acquire().await?;

        // Get the existing quote from database
        // TODO: keep a record of our fees somewhere
        let (unit, required_amount, _fee, state, expiry, _quote_hash, payment_request) =
            db_node::melt_quote::get_data::<Unit>(&mut conn, quote_id).await?;

        // Check if quote is still valid
        if expiry < unix_time() {
            return Err(Error::QuoteExpired(quote_id));
        }

        // Check if quote is in correct state
        if state != nuts::nut05::MeltQuoteState::Unpaid {
            return Err(Error::QuoteAlreadyProcessed(quote_id));
        }

        // Process and validate inputs
        let mut tx = db_node::start_db_tx_from_conn(&mut conn)
            .await
            .map_err(Error::TxBegin)?;

        let (total_amount, insert_spent_proof_query) = process_melt_inputs(
            &mut tx,
            self.signer.clone(),
            self.keyset_cache.clone(),
            inputs,
            unit,
        )
        .await?;

        // Verify the input amount matches the quote amount
        if total_amount != required_amount {
            return Err(Error::InvalidAmount(total_amount, required_amount));
        }

        // Mark inputs as spent
        insert_spent_proof_query.execute(&mut tx).await?;
        tx.commit().await?;

        // Process the actual payment
        let state = {
            // Get withdrawer and deserialize payment request
            let mut withdrawer = self
                .liquidity_sources
                .get_liquidity_source(method)
                .ok_or(Error::MethodNotSupported(method))?
                .withdrawer();

            // Deserialize the payment request
            let payment_request = withdrawer
                .deserialize_payment_request(&payment_request)
                .map_err(|e| Error::LiquiditySource(e.into()))?;

            withdrawer
                .proceed_to_payment(quote_id, payment_request, expiry)
                .await
                .map_err(|e| Error::LiquiditySource(e.into()))?
        };

        // Update quote state and transfer ID
        db_node::melt_quote::set_state(&mut conn, quote_id, state).await?;

        let meter = opentelemetry::global::meter("business");
        let n_melt_counter = meter.u64_counter("melt.operation.count").build();
        n_melt_counter.add(1, &[]);

        event!(
            name: "melt",
            Level::INFO,
            name = "melt",
            %method,
            %quote_id,
        );

        // Wait until the paiment events have been indexed
        loop {
            if let (MeltQuoteState::Paid, transfer_ids) =
                db_node::melt_quote::get_state_and_transfer_ids(&mut conn, quote_id).await?
            {
                return Ok(transfer_ids);
            } else {
                let _ = tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
