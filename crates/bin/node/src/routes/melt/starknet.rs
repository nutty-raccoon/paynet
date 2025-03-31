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
            &settings,
            self.quote_ttl.melt_ttl(),
            raw_melt_payment_request,
            inputs,
        )
        .await?;

        let amount_to_pay = settings.unit.convert_amount_into_u256(total_amount);

        #[cfg(not(feature = "starknet"))]
        let (state, tx_hash) = proceed_to_payment(&mut conn, quote_id, quote_hash).await?;
        #[cfg(feature = "starknet")]
        let (state, tx_hash) = proceed_to_payment(
            &mut conn,
            quote_id,
            quote_hash,
            melt_payment_request,
            amount_to_pay,
            self.starknet_config.cashier.clone(),
        )
        .await?;
        db_node::melt_quote::register_transfer_id(&mut conn, quote_id, &tx_hash).await?;

        Ok(MeltQuoteResponse {
            quote: quote_id,
            amount: total_amount,
            fee,
            state,
            expiry,
            transfer_id: tx_hash,
        })
    }
}

#[cfg(not(feature = "starknet"))]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
    _quote_hash: Sha256,
) -> Result<(MeltQuoteState, Vec<u8>), Error> {
    let new_state = MeltQuoteState::Paid;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok(new_state, b"cafebabe")
}

#[cfg(feature = "starknet")]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
    quote_hash: Sha256,
    melt_payment_request: MeltPaymentRequest,
    amount: starknet_types::StarknetU256,
    mut starknet_cashier: crate::app_state::starknet::StarknetCashierClient,
) -> Result<(MeltQuoteState, Vec<u8>), Error> {
    use starknet_cashier::WithdrawRequest;
    use tonic::Request;

    let tx_hash = starknet_cashier
        .withdraw(Request::new(WithdrawRequest {
            invoice_id: quote_hash.to_byte_array().to_vec(),
            asset: melt_payment_request.asset.to_string(),
            amount: amount
                .to_bytes_be()
                .into_iter()
                .skip_while(|&b| b == 0)
                .collect(),
            payee: melt_payment_request.payee.to_bytes_be().to_vec(),
        }))
        .await
        .map_err(Error::StarknetCashier)?
        .into_inner()
        .tx_hash;
    let new_state = MeltQuoteState::Pending;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok((new_state, tx_hash))
}
