use std::str::FromStr;
use tauri_plugin_opener::OpenerExt;
use tracing::error;

use nuts::nut04::MintQuoteState;
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, STARKNET_STR};
use tauri::{AppHandle, State};

use crate::errors::CommonError;
use crate::front_events::{
    MintQuoteCreatedEvent, PendingMintQuoteData, emit_mint_quote_created_event,
};
use crate::mint_quote::Node;
use crate::{AppState, mint_quote::MintQuoteStateMachine};
use parse_asset_amount::{ParseAmountStringError, parse_asset_amount};

#[derive(Debug, thiserror::Error)]
pub enum CreateMintQuoteError {
    #[error(transparent)]
    Common(#[from] crate::errors::CommonError),
    #[error("failed to parse asset: {0}")]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error("failed to convert asset to unit: {0}")]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error("failed to redeem quote: {0}")]
    Redeem(#[from] RedeemQuoteError),
    #[error("failed pay the quote: {0}")]
    Pay(#[from] PayQuoteError),
}

impl serde::Serialize for CreateMintQuoteError {
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
pub async fn create_mint_quote(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: u32,
    amount: String,
    asset: String,
) -> Result<(), CreateMintQuoteError> {
    let asset = Asset::from_str(&asset)?;
    let unit = asset.find_best_unit();
    let amount = parse_asset_amount(&amount, asset, unit)?;

    let node_url = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::NodeId(node_id))?
    };
    let mut node_client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert())
        .await
        .map_err(CommonError::CreateNodeClient)?;

    let response = wallet::mint::create_quote(
        state.pool.clone(),
        &mut node_client,
        node_id,
        STARKNET_STR.to_string(),
        amount,
        unit,
    )
    .await
    .map_err(CommonError::Wallet)?;

    emit_mint_quote_created_event(
        &app,
        MintQuoteCreatedEvent {
            node_id,
            mint_quote: PendingMintQuoteData {
                id: response.quote.clone(),
                unit: unit.to_string(),
                amount: amount.into(),
            },
        },
    )
    .map_err(CommonError::EmitTauriEvent)?;

    let node = Node {
        id: node_id,
        client: node_client,
    };

    inner_pay_quote(&app, &state, node, response.quote, response.request).await?;

    Ok(())
}

#[tauri::command]
pub async fn pay_quote(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: u32,
    quote_id: String,
) -> Result<(), PayQuoteError> {
    let node_url = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::NodeId(node_id))?
    };

    let mint_quote = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::mint_quote::get(&db_conn, node_id, &quote_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::QuoteNotFound(quote_id.clone()))?
    };

    let node_client = wallet::connect_to_node_lazy(&node_url, state.opt_root_ca_cert())
        .map_err(CommonError::CreateNodeClient)?;

    let node = Node {
        id: node_id,
        client: node_client,
    };

    inner_pay_quote(&app, &state, node, quote_id, mint_quote.request).await
}

#[derive(Debug, thiserror::Error)]
pub enum RedeemQuoteError {
    #[error(transparent)]
    Common(#[from] crate::errors::CommonError),
    #[error("quote not paid")]
    QuoteNotPaid,
    #[error("failed parse db unit: {0}")]
    Unit(#[from] starknet_types::UnitFromStrError),
}

impl serde::Serialize for RedeemQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PayQuoteError {
    #[error(transparent)]
    Common(#[from] crate::errors::CommonError),
    #[error("failed to deposit payload: {0}")]
    ParseDepositPayload(serde_json::Error),
    #[error("failed to deposit calldatas: {0}")]
    SerializeCalldata(serde_json::Error),
    #[error("failed to open the link for paying the invoice: {0}")]
    OpenLink(#[from] tauri_plugin_opener::Error),
}

impl serde::Serialize for PayQuoteError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn redeem_quote(
    state: State<'_, AppState>,
    node_id: u32,
    quote_id: String,
) -> Result<(), RedeemQuoteError> {
    let node_url = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::NodeId(node_id))?
    };

    let mint_quote = {
        let db_conn = state.pool.get().map_err(CommonError::DbPool)?;
        wallet::db::mint_quote::get(&db_conn, node_id, &quote_id)
            .map_err(CommonError::Db)?
            .ok_or(CommonError::QuoteNotFound(quote_id.clone()))?
    };

    if mint_quote.state != MintQuoteState::Paid {
        return Err(RedeemQuoteError::QuoteNotPaid);
    }

    let node_client = wallet::connect_to_node_lazy(&node_url, state.opt_root_ca_cert())
        .map_err(CommonError::CreateNodeClient)?;

    state
        .mint_quote_event_sender
        .send(MintQuoteStateMachine::Paid {
            node: Node {
                id: node_id,
                client: node_client,
            },
            quote_id,
        })
        .await
        .map_err(|_| CommonError::MintQuoteChannel)?;

    Ok(())
}

async fn inner_pay_quote(
    app: &AppHandle,
    state: &AppState,
    node: Node,
    quote_id: String,
    request: String,
) -> Result<(), PayQuoteError> {
    // Mock nodes return empty request for deposit
    // Only allowed in debug mode
    #[cfg(debug_assertions)]
    if request.is_empty() {
        state
            .mint_quote_event_sender
            .send(MintQuoteStateMachine::Paid { node, quote_id })
            .await
            .map_err(|_| CommonError::MintQuoteChannel)?;
        return Ok(());
    } else {
        state
            .mint_quote_event_sender
            .send(MintQuoteStateMachine::Created { node, quote_id })
            .await
            .map_err(|_| CommonError::MintQuoteChannel)?;
    }

    let deposit_payload: starknet_types::DepositPayload =
        serde_json::from_str(&request).map_err(PayQuoteError::ParseDepositPayload)?;
    let payload_json = serde_json::to_string(&deposit_payload.call_data)
        .map_err(PayQuoteError::SerializeCalldata)?;
    let encoded_payload = urlencoding::encode(&payload_json);

    // On desktop we open the browser
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    let url = format!(
        "{}/deposit/{}/{}/?payload={}",
        &state.web_app_url,
        STARKNET_STR,
        deposit_payload.chain_id.as_str(),
        encoded_payload
    );
    app.opener().open_url(url, None::<&str>)?;

    Ok(())
}
