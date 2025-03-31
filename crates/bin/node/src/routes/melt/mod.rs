mod errors;
#[cfg(not(feature = "starknet"))]
mod mock;
#[cfg(feature = "starknet")]
mod starknet;

use bitcoin_hashes::Sha256;
use nuts::Amount;
use nuts::nut05::{MeltQuoteResponse, MeltQuoteState};
use nuts::{nut00::Proof, nut05::MeltMethodSettings};
use sqlx::PgConnection;
use starknet_types::{Asset, Unit};
use uuid::Uuid;

use crate::logic::process_melt_inputs;
use crate::utils::unix_time;
use crate::{grpc_service::GrpcState, methods::Method};

pub trait PaymentRequest {
    fn asset(&self) -> Asset;
}

#[async_trait::async_trait]
pub trait MeltBackend {
    type PaymentRequest: std::fmt::Debug
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + PaymentRequest
        + Send;

    fn deserialize_payment_request(
        &self,
        raw_json_string: &str,
    ) -> Result<Self::PaymentRequest, Error>;

    async fn proceed_to_payment(
        &mut self,
        quote_hash: Sha256,
        payment_request: Self::PaymentRequest,
        unit: Unit,
        amount: Amount,
    ) -> Result<(MeltQuoteState, Vec<u8>), Error>;
}

use errors::Error;

impl GrpcState {
    fn get_backend(&self, method: Method) -> Result<impl MeltBackend, Error> {
        match method {
            Method::Starknet => {
                #[cfg(not(feature = "starknet"))]
                return Ok(mock::MockMeltBackend);

                #[cfg(feature = "starknet")]
                return Ok(starknet::StarknetMeltBackend(
                    self.starknet_config.cashier.clone(),
                ));
            }
        }
    }

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

        let mut backend = self.get_backend(method)?;
        let payment_request = backend.deserialize_payment_request(&melt_payment_request)?;
        let asset = payment_request.asset();
        if !settings.unit.is_asset_supported(asset) {
            return Err(Error::InvalidAssetForUnit(asset, settings.unit));
        }

        let mut conn = self.pg_pool.acquire().await?;

        let (quote_id, quote_hash, total_amount, fee, expiry) = self
            .validate_and_register_quote(&mut conn, &settings, melt_payment_request, inputs)
            .await?;
        let (state, transfer_id) = backend
            .proceed_to_payment(quote_hash, payment_request, settings.unit, total_amount)
            .await?;
        // TODO: merge those in a signle call
        db_node::melt_quote::set_state(&mut conn, quote_id, state).await?;
        db_node::melt_quote::register_transfer_id(&mut conn, quote_id, &transfer_id).await?;

        Ok(MeltQuoteResponse {
            quote: quote_id,
            amount: total_amount,
            fee,
            state,
            expiry,
            transfer_id,
        })
    }

    async fn validate_and_register_quote(
        &self,
        conn: &mut PgConnection,
        settings: &MeltMethodSettings<Method, Unit>,
        melt_payment_request: String,
        inputs: &[Proof],
    ) -> Result<(Uuid, bitcoin_hashes::Sha256, Amount, Amount, u64), Error> {
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

        let expiry = unix_time() + self.quote_ttl.melt_ttl();
        let quote = Uuid::new_v4();
        let quote_hash = bitcoin_hashes::Sha256::hash(quote.as_bytes());
        // Arbitrary for now, but will be enough to pay tx fee on starknet
        let fee = Amount::ONE;

        db_node::melt_quote::insert_new(
            &mut tx,
            quote,
            quote_hash.as_byte_array(),
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

        Ok((quote, quote_hash, total_amount, fee, expiry))
    }
}
