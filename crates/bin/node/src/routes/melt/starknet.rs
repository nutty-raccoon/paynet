use bitcoin_hashes::Sha256;
use nuts::{
    nut00::Proof,
    nut05::{MeltMethodSettings, MeltQuoteResponse, MeltQuoteState},
};
use sqlx::PgConnection;
use starknet_types::{MeltPaymentRequest, Unit};
use uuid::Uuid;

use crate::{grpc_service::GrpcState, methods::Method};

use super::{errors::Error, validate_and_register_quote};

impl GrpcState {
    pub async fn starknet_melt(
        &self,
        settings: MeltMethodSettings<Method, Unit>,
        raw_melt_payment_request: String,
        inputs: &[Proof],
    ) -> Result<MeltQuoteResponse<Uuid>, Error> {
        let mut conn = self.pg_pool.acquire().await?;

        let melt_payment_request: MeltPaymentRequest =
            serde_json::from_str(&raw_melt_payment_request)
                .map_err(Error::InvalidPaymentRequest)?;

        if melt_payment_request.recipient == starknet_types_core::felt::Felt::ZERO {
            return Err(Error::InvalidPaymentRequest(serde_json::Error::custom(
                "Recipient address cannot be zero",
            )));
        }

        if melt_payment_request.recipient.to_bytes_be()[0] != 0 {
            return Err(Error::InvalidPaymentRequest(serde_json::Error::custom(
                "Recipient address is too large",
            )));
        }

        let hex = melt_payment_request.recipient.to_hex_string();
        if hex.is_empty() {
            return Err(Error::InvalidPaymentRequest(serde_json::Error::custom(
                "Recipient address is not properly formatted",
            )));
        }

        if !settings.unit.is_asset_supported(melt_payment_request.asset) {
            return Err(Error::InvalidAssetForUnit(
                melt_payment_request.asset,
                settings.unit,
            ));
        }

        let (quote_id, quote_hash, total_amount, fee, expiry) = validate_and_register_quote(
            &mut conn,
            self.signer.clone(),
            self.keyset_cache.clone(),
            settings,
            self.quote_ttl.melt_ttl(),
            raw_melt_payment_request,
            inputs,
        )
        .await?;

        #[cfg(not(feature = "starknet"))]
        let state = proceed_to_payment(&mut conn, quote_id, quote_hash).await?;
        #[cfg(feature = "starknet")]
        let state = proceed_to_payment(
            &mut conn,
            quote_id,
            quote_hash,
            melt_payment_request,
            total_amount,
            self.starknet_cashier.clone(),
        )
        .await?;

        Ok(MeltQuoteResponse {
            quote: quote_id,
            amount: total_amount,
            fee,
            state,
            expiry,
        })
    }
}

#[cfg(not(feature = "starknet"))]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
    _quote_hash: Sha256,
) -> Result<MeltQuoteState, Error> {
    let new_state = MeltQuoteState::Paid;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok(new_state)
}

#[cfg(feature = "starknet")]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
    quote_hash: Sha256,
    melt_payment_request: MeltPaymentRequest,
    amount: nuts::Amount,
    mut starknet_cashier: crate::app_state::StarknetCashierClient,
) -> Result<MeltQuoteState, Error> {
    use starknet_cashier::WithdrawRequest;
    use tonic::Request;

    starknet_cashier
        .withdraw(Request::new(WithdrawRequest {
            amount: u64::from(amount)
                .to_be_bytes()
                .into_iter()
                .skip_while(|&b| b == 0)
                .collect(),
            asset: melt_payment_request.asset.to_string(),
            invoice_id: quote_hash.to_byte_array().to_vec(),
            payee: melt_payment_request
                .recipient
                .to_bytes_be()
                .into_iter()
                .skip_while(|&b| b == 0)
                .collect(),
        }))
        .await
        .map_err(Error::StarknetCashier)?;
    let new_state = MeltQuoteState::Pending;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok(new_state)
}
