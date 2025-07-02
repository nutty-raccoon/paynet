use liquidity_source::WithdrawInterface;
use num_traits::CheckedAdd;
use nuts::{Amount, nut05::MeltQuoteState};
use starknet_types::{Asset, AssetToUnitConversionError, Unit, is_valid_starknet_address};
use starknet_types_core::felt::Felt;
use uuid::Uuid;

use crate::StarknetInvoiceId;

use super::MeltPaymentRequest;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid payment request json string: {0}")]
    InvalidPaymentRequest(#[from] serde_json::Error),
    #[error("invalid starknet address: {0}")]
    InvalidStarknetAddress(Felt),
    #[error("amount overflow")]
    Overflow,
    #[error("unsupported asset `{0}` for unit `{1}`")]
    InvalidAssetForUnit(Asset, Unit),
    #[error("failed to convert request values to nodes values: {0}")]
    Conversion(#[from] AssetToUnitConversionError),
}

#[derive(Debug, Clone)]
pub struct Withdrawer;

#[async_trait::async_trait]
impl WithdrawInterface for Withdrawer {
    type Error = Error;
    type Request = MeltPaymentRequest;
    type Unit = Unit;
    type InvoiceId = StarknetInvoiceId;

    fn deserialize_payment_request(&self, raw_json_string: &str) -> Result<Self::Request, Error> {
        let pr = serde_json::from_str::<Self::Request>(raw_json_string)
            .map_err(Error::InvalidPaymentRequest)?;

        if !is_valid_starknet_address(&pr.payee) {
            return Err(Error::InvalidStarknetAddress(pr.payee));
        }

        Ok(pr)
    }

    fn compute_total_amount_expected(
        &self,
        request: Self::Request,
        unit: Unit,
        fee: Amount,
    ) -> Result<nuts::Amount, Self::Error> {
        if !unit.is_asset_supported(request.asset) {
            return Err(Error::InvalidAssetForUnit(request.asset, unit));
        }

        let (amount, rem) = request
            .asset
            .convert_to_amount_of_unit(request.amount.clone().into(), unit)?;

        if fee == Amount::ZERO {
            if rem.is_zero() {
                Ok(amount)
            } else {
                amount.checked_add(&Amount::ONE).ok_or(Error::Overflow)
            }
        } else {
            amount.checked_add(&fee).ok_or(Error::Overflow)
        }
    }

    async fn proceed_to_payment(
        &mut self,
        _quote_id: Uuid,
        _melt_payment_request: MeltPaymentRequest,
        _expiry: u64,
    ) -> Result<MeltQuoteState, Error> {
        Ok(MeltQuoteState::Paid)
    }
}
