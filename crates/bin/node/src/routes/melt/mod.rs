mod errors;
mod inputs;

use inputs::process_melt_inputs;
use liquidity_source::{LiquiditySource, WithdrawAmount, WithdrawInterface, WithdrawRequest};
use nuts::Amount;
use nuts::nut05::MeltQuoteResponse;
use nuts::{nut00::Proof, nut05::MeltMethodSettings};
use sqlx::PgConnection;
use starknet_types::Unit;
use tracing::{Level, event};
use uuid::Uuid;

use crate::utils::unix_time;
use crate::{grpc_service::GrpcState, methods::Method};

use errors::Error;

impl GrpcState {
    pub async fn inner_melt(
        &self,
        method: Method,
        unit: Unit,
        melt_payment_request: String,
        inputs: &[Proof],
    ) -> Result<MeltQuoteResponse<Uuid>, Error> {
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
        let mut withdrawer = liquidity_source.withdrawer();

        let payment_request = withdrawer
            .deserialize_payment_request(&melt_payment_request)
            .map_err(|e| Error::LiquiditySource(e.into()))?;
        let asset = payment_request.asset();
        if !settings.unit.is_asset_supported(asset) {
            return Err(Error::InvalidAssetForUnit(asset, settings.unit));
        }
        let expiry = unix_time() + self.quote_ttl.melt_ttl();
        let quote_id = Uuid::new_v4();
        let invoice_id = liquidity_source.compute_invoice_id(quote_id, expiry);

        let mut conn = self.pg_pool.acquire().await?;

        let (total_amount, fee) = self
            .validate_and_register_quote(
                &mut conn,
                &settings,
                melt_payment_request,
                inputs,
                quote_id,
                invoice_id.clone(),
                expiry,
            )
            .await?;

        event!(
            name: "melt-quote",
            Level::INFO,
            name = "melt-quote",
            %method,
            amount = u64::from(total_amount),
            %unit,
            %quote_id,
        );

        let state = withdrawer
            .proceed_to_payment(
                quote_id,
                payment_request,
                WithdrawAmount::convert_from(settings.unit, total_amount),
                expiry,
            )
            .await
            .map_err(|e| Error::LiquiditySource(e.into()))?;

        db_node::melt_quote::set_state(&mut conn, quote_id, state).await?;

        event!(
            name: "melt",
            Level::INFO,
            name = "melt",
            %method,
            %quote_id,
        );

        let meter = opentelemetry::global::meter("business");
        let n_melt_counter = meter.u64_counter("melt.operation.count").build();
        n_melt_counter.add(1, &[]);

        Ok(MeltQuoteResponse {
            quote: quote_id,
            amount: total_amount,
            fee,
            state,
            expiry,
            transfer_ids: None,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn validate_and_register_quote(
        &self,
        conn: &mut PgConnection,
        settings: &MeltMethodSettings<Method, Unit>,
        melt_payment_request: String,
        inputs: &[Proof],
        quote_id: Uuid,
        invoice_id: impl Into<[u8; 32]>,
        expiry: u64,
    ) -> Result<(Amount, Amount), Error> {
        let mut tx = db_node::start_db_tx_from_conn(conn)
            .await
            .map_err(Error::TxBegin)?;

        let (total_amount, insert_spent_proof_query) = process_melt_inputs(
            &mut tx,
            self.signer.clone(),
            self.keyset_cache.clone(),
            inputs,
        )
        .await?;

        if let Some(min_amount) = settings.min_amount {
            if min_amount > total_amount {
                Err(Error::AmountTooLow(total_amount, min_amount))?;
            }
        }
        if let Some(max_amount) = settings.max_amount {
            if max_amount < total_amount {
                Err(Error::AmountTooHigh(max_amount, total_amount))?;
            }
        }

        // Arbitrary for now, but will be enough to pay tx fee on starknet
        let fee = Amount::ONE;

        db_node::melt_quote::insert_new(
            &mut tx,
            quote_id,
            &invoice_id.into(),
            settings.unit,
            total_amount,
            fee,
            &serde_json::to_string(&melt_payment_request)
                .expect("it has been deserialized it should be serializable"),
            expiry,
        )
        .await?;
        insert_spent_proof_query.execute(&mut tx).await?;

        tx.commit().await?;

        Ok((total_amount, fee))
    }
}
