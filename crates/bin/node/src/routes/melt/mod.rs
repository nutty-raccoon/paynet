mod errors;
mod starknet;
use nuts::Amount;
use nuts::nut05::MeltQuoteResponse;
use nuts::{nut00::Proof, nut05::MeltMethodSettings};
use sqlx::PgConnection;
use starknet_types::Unit;
use uuid::Uuid;

use crate::logic::process_melt_inputs;
use crate::utils::unix_time;
use crate::{
    app_state::SignerClient, grpc_service::GrpcState, keyset_cache::KeysetCache, methods::Method,
};

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

        let response = match method {
            Method::Starknet => {
                self.starknet_melt(settings, melt_payment_request, inputs)
                    .await?
            }
        };

        Ok(response)
    }
}

async fn validate_and_register_quote(
    conn: &mut PgConnection,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    settings: &MeltMethodSettings<Method, Unit>,
    mint_ttl: u64,
    melt_payment_request: String,
    inputs: &[Proof],
) -> Result<(Uuid, bitcoin_hashes::Sha256, Amount, Amount, u64), Error> {
    let mut tx = db_node::start_db_tx_from_conn(conn)
        .await
        .map_err(Error::TxBegin)?;

    let (total_amount, insert_spent_proof_query) =
        process_melt_inputs(&mut tx, signer, keyset_cache, inputs).await?;

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

    let expiry = unix_time() + mint_ttl;
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
