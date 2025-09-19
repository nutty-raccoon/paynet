use std::str::FromStr;
use tracing::{Level, error, event};

use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, STARKNET_STR};
use tauri::{AppHandle, State};

use crate::AppState;
use crate::errors::CommonError;
use crate::front_events::emit_trigger_pending_quote_poll;
use crate::quote_handler::{MeltQuoteAction, QuoteHandlerEvent};
use parse_asset_amount::{ParseAmountStringError, parse_asset_amount};

#[derive(Debug, thiserror::Error)]
pub enum CreateMeltQuoteError {
    #[error(transparent)]
    Common(#[from] crate::errors::CommonError),
    #[error("failed to parse asset: {0}")]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error("failed to convert asset to unit: {0}")]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error("starknet error: {0}")]
    Starknet(#[from] StarknetError),
    #[error("non supported method: {0}")]
    Method(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StarknetError {
    #[error("failed to create melt quote request: {0}")]
    CreateRequest(#[from] starknet_liquidity_source::NewMeltPaymentRequestError),
    #[error("failed to serialize payment request: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl serde::Serialize for CreateMeltQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl serde::Serialize for StarknetError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMintQuoteResponse {
    quote_id: String,
    payment_request: String,
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub async fn create_melt_quote(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: u32,
    method: String,
    amount: String,
    asset: String,
    to: String,
) -> Result<(), CreateMeltQuoteError> {
    let asset = Asset::from_str(&asset)?;
    let unit = asset.find_best_unit();
    let amount = parse_asset_amount(&amount, asset, unit)?;
    let on_chain_amount = unit.convert_amount_into_u256(amount);

    let serialized_payment_request = if method == STARKNET_STR {
        let request =
            starknet_liquidity_source::MeltPaymentRequest::new(to, asset, on_chain_amount.into())
                .map_err(StarknetError::CreateRequest)?;
        serde_json::to_string(&request).map_err(StarknetError::SerdeJson)?
    } else {
        return Err(CreateMeltQuoteError::Method(method));
    };

    let mut node_client = state
        .get_node_client_connection(node_id)
        .await
        .map_err(CommonError::CachedConnection)?;

    event!(name: "creating_melt_quote", Level::INFO,
        node_id = node_id,
        method = %method,
        unit = %unit,
        "Creating melt quote"
    );

    let response = wallet::melt::create_quote(
        state.pool.clone(),
        &mut node_client,
        node_id,
        method.clone(),
        unit.to_string(),
        serialized_payment_request,
    )
    .await
    .map_err(CommonError::Wallet)?;

    event!(name: "melt_quote_created", Level::INFO,
        node_id = node_id,
        quote_id = response.quote,
        method,
        %unit,
        %amount,
        "Melt quote created"
    );

    let _ = emit_trigger_pending_quote_poll(&app);

    Ok(())
}

#[tauri::command]
#[tracing::instrument(skip(state))]
pub async fn pay_melt_quote(
    state: State<'_, AppState>,
    node_id: u32,
    quote_id: String,
) -> Result<(), PayQuoteError> {
    state
        .quote_event_sender
        .send(QuoteHandlerEvent::Melt(MeltQuoteAction::Pay {
            node_id,
            quote_id,
        }))
        .await
        .map_err(|_| CommonError::QuoteHandlerChannel)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PayQuoteError {
    #[error(transparent)]
    Common(#[from] crate::errors::CommonError),
}

impl serde::Serialize for PayQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
