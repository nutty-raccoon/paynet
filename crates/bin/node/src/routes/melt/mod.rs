mod errors;
mod inputs;

use bitcoin_hashes::Sha256;
use inputs::process_melt_inputs;
use liquidity_source::{LiquiditySource, WithdrawAmount, WithdrawInterface, WithdrawRequest};
use nuts::Amount;
use nuts::nut00::Proof;
use starknet_types::Unit;
use uuid::Uuid;

use crate::routes::melt;
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
    ) -> Result<nuts::nut05::MeltQuoteResponse<Uuid>, Error> {
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

        let withdrawer = self
            .liquidity_sources
            .get_liquidity_source(method)
            .ok_or(Error::MethodNotSupported(method))?
            .withdrawer();

        // Validate the payment request format
        let payment_request = withdrawer
            .deserialize_payment_request(&melt_payment_request)
            .map_err(|e| Error::LiquiditySource(e.into()))?;
        let asset: starknet_types::Asset = payment_request.asset();

        if !settings.unit.is_asset_supported(asset) {
            return Err(Error::InvalidAssetForUnit(asset, settings.unit));
        }

        let expiry = unix_time() + self.quote_ttl.melt_ttl();
        let quote_id = Uuid::new_v4();
        let amount = payment_request.amount(); // Get amount from payment request
        let quote_hash = bitcoin_hashes::Sha256::hash(quote_id.as_bytes());

        // Arbitrary fee for now, but will be enough to pay tx fee on starknet
        let fee = Amount::ONE;

        // Store the quote in database
        let mut conn = self.pg_pool.acquire().await?;
        db_node::melt_quote::insert_new(
            &mut conn,
            quote_id,
            quote_hash.as_byte_array(),
            settings.unit,
            amount,
            fee,
            &melt_payment_request,
            expiry,
        )
        .await?;

        Ok(nuts::nut05::MeltQuoteResponse {
            quote: quote_id,
            amount,
            fee,
            state: nuts::nut05::MeltQuoteState::Unpaid,
            expiry,
            transfer_id: vec![],
        })
    }

    /// Step 2: Execute the melt using an existing quote ID
    /// This processes the actual payment using the previously created quote
    pub async fn inner_melt(
        &self,
        method: Method,
        quote_id: Uuid,
        inputs: &[Proof],
    ) -> Result<nuts::nut05::MeltQuoteResponse<Uuid>, Error> {
        let mut conn = self.pg_pool.acquire().await?;

        // Get the existing quote from database
        let (unit, amount, fee, state, expiry, quote_hash, payment_request) =
            db_node::melt_quote::get_data(&mut conn, quote_id).await?;

        // Check if quote is still valid
        if expiry < unix_time() {
            return Err(Error::QuoteExpired(quote_id));
        }

        // Check if quote is in correct state
        if state != nuts::nut05::MeltQuoteState::Unpaid {
            return Err(Error::QuoteAlreadyProcessed(quote_id));
        }

        // Get settings for this quote
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

        // Process and validate inputs
        let mut tx = db_node::start_db_tx_from_conn(&mut conn)
            .await
            .map_err(Error::TxBegin)?;

        let (total_amount, insert_spent_proof_query) = process_melt_inputs(
            &mut tx,
            self.signer.clone(),
            self.keyset_cache.clone(),
            inputs,
        )
        .await?;

        // Verify the input amount matches the quote amount + fee
        let required_amount = amount + fee;
        if total_amount != required_amount {
            return Err(Error::InvalidAmount(total_amount, required_amount));
        }

        // Mark inputs as spent
        insert_spent_proof_query.execute(&mut tx).await?;
        tx.commit().await?;

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
        // Process the actual payment
        let (state, transfer_id) = withdrawer
            .proceed_to_payment(
                Sha256::from_byte_array(quote_hash),
                payment_request,
                WithdrawAmount::convert_from(settings.unit, amount),
                expiry,
            )
            .await
            .map_err(|e| Error::LiquiditySource(e.into()))?;

        // Update quote state and transfer ID
        db_node::melt_quote::set_state(&mut conn, quote_id, state).await?;
        db_node::melt_quote::register_transfer_id(&mut conn, quote_id, &transfer_id).await?;

        Ok(nuts::nut05::MeltQuoteResponse {
            quote: quote_id,
            amount: amount,
            fee: fee,
            state,
            expiry: expiry,
            transfer_id: transfer_id.to_vec(),
        })
    }
}
