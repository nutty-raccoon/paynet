use node_client::NodeClient;
use std::str::FromStr;
use tauri_plugin_opener::OpenerExt;

use nuts::traits::Unit as UnitT;
use nuts::{Amount, nut04::MintQuoteState};
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, STARKNET_STR, Unit};
use tauri::{AppHandle, Emitter, State};
use tonic::transport::Channel;

use crate::{AppState, commands::BalanceChange};
use parse_asset_amount::{ParseAmountStringError, parse_asset_amount};

#[derive(Debug, thiserror::Error)]
pub enum CreateMintQuoteError {
    #[error("failed to get a connection from the pool: {0}")]
    R2D2(#[from] r2d2::Error),
    #[error("failed to interact with database: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("failed wallet logic: {0}")]
    Wallet(#[from] wallet::errors::Error),
    #[error("unknown node_id: {0}")]
    NodeId(u32),
    #[error("failed to parse asset: {0}")]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error("failed to convert asset to unit: {0}")]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error(transparent)]
    ConnectToNode(#[from] wallet::ConnectToNodeError),
    #[error("failed to deposit payload: {0}")]
    ParseDepositPayload(serde_json::Error),
    #[error("failed to deposit calldatas: {0}")]
    SerializeCalldata(serde_json::Error),
    #[error("failed to redeem quote")]
    Redeem(#[from] RedeemQuoteError),
    #[error("failed to open the link for paying the invoice: {0}")]
    OpenLink(#[from] tauri_plugin_opener::Error),
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
        let db_conn = state.pool.get()?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(CreateMintQuoteError::NodeId(node_id))?
    };
    let mut node_client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert()).await?;

    let response = wallet::mint::create_quote(
        state.pool.clone(),
        &mut node_client,
        node_id,
        STARKNET_STR.to_string(),
        amount,
        unit,
    )
    .await?;

    // Mock nodes return empty request for deposit
    // Only allowed in debug mode
    #[cfg(debug_assertions)]
    if response.request.is_empty() {
        inner_redeem_quote(
            &app,
            state.clone(),
            &mut node_client,
            node_id,
            &response.quote,
            unit.to_string(),
            amount,
        )
        .await?;

        return Ok(());
    }

    let deposit_payload: starknet_types::DepositPayload =
        serde_json::from_str(&response.request)
            .map_err(CreateMintQuoteError::ParseDepositPayload)?;
    let payload_json = serde_json::to_string(&deposit_payload.call_data)
        .map_err(CreateMintQuoteError::SerializeCalldata)?;
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
    log::info!("---- url: {:?}", url);
    app.opener().open_url(url, None::<&str>)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RedeemQuoteError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error("unknown node_id: {0}")]
    NodeId(u32),
    #[error("quote not paid")]
    QuoteNotPaid,
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    NodeConnect(#[from] wallet::ConnectToNodeError),
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

#[tauri::command]
pub async fn redeem_quote(
    app: AppHandle,
    state: State<'_, AppState>,
    node_id: u32,
    quote_id: String,
) -> Result<(), RedeemQuoteError> {
    let node_url = {
        let db_conn = state.pool.get()?;
        wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .ok_or(RedeemQuoteError::NodeId(node_id))?
    };
    let mut node_client = wallet::connect_to_node(&node_url, state.opt_root_ca_cert()).await?;

    let mint_quote = {
        let db_conn = state.pool.get()?;
        wallet::db::mint_quote::get(&db_conn, node_id, &quote_id)?
    };

    if mint_quote.state != MintQuoteState::Paid {
        return Err(RedeemQuoteError::QuoteNotPaid);
    }

    inner_redeem_quote(
        &app,
        state,
        &mut node_client,
        node_id,
        &quote_id,
        mint_quote.unit,
        mint_quote.amount,
    )
    .await
}

pub(crate) async fn inner_redeem_quote(
    app: &AppHandle,
    state: State<'_, AppState>,
    node_client: &mut NodeClient<Channel>,
    node_id: u32,
    quote_id: &str,
    unit: String,
    amount: Amount,
) -> Result<(), RedeemQuoteError> {
    wallet::mint::redeem_quote(
        crate::SEED_PHRASE_MANAGER,
        state.pool.clone(),
        node_client,
        STARKNET_STR.to_string(),
        quote_id,
        node_id,
        unit.as_str(),
        amount,
    )
    .await?;

    app.emit(
        "balance-increase",
        BalanceChange {
            node_id,
            unit: unit.as_str().to_string(),
            amount: amount.into(),
        },
    )?;

    state
        .get_prices_config
        .write()
        .await
        .assets
        .insert(Unit::from_str(&unit)?.matching_asset());

    Ok(())
}
