use std::str::FromStr;
use tauri_plugin_opener::OpenerExt;

use nuts::nut04::MintQuoteState;
use nuts::traits::Unit as UnitT;
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, STARKNET_STR, Unit};
use tauri::{AppHandle, Emitter, State};

use crate::{AppState, commands::BalanceChange};
use parse_asset_amount::{ParseAmountStringError, parse_asset_amount};

#[derive(Debug, thiserror::Error)]
pub enum CreateMintQuoteError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error("unknown node_id: {0}")]
    NodeId(u32),
    #[error(transparent)]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error(transparent)]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error(transparent)]
    ConnectToNode(#[from] wallet::ConnectToNodeError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
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
) -> Result<CreateMintQuoteResponse, CreateMintQuoteError> {
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

    let deposit_payload: starknet_types::DepositPayload =
        serde_json::from_str(&response.request)?;
    let payload_json = serde_json::to_string(&deposit_payload.call_data)?;
    let encoded_payload = urlencoding::encode(&payload_json);

    let url = format!(
        "{}/deposit/{}/{}/?payload={}",
        &state.web_app_url,
        STARKNET_STR,
        deposit_payload.chain_id.as_str(),
        encoded_payload
    );
    log::info!("---- url: {:?}", url);
    let x = app.opener().open_url(url, None::<&str>);
    log::info!("---- openner result: {:?}", x);

    Ok(CreateMintQuoteResponse {
        quote_id: response.quote,
        payment_request: response.request,
    })
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

    wallet::mint::redeem_quote(
        crate::SEED_PHRASE_MANAGER,
        state.pool.clone(),
        &mut node_client,
        STARKNET_STR.to_string(),
        mint_quote.id,
        node_id,
        &mint_quote.unit,
        mint_quote.amount,
    )
    .await?;

    let unit = Unit::from_str(&mint_quote.unit)?;
    app.emit(
        "balance-increase",
        BalanceChange {
            node_id,
            unit: mint_quote.unit,
            amount: mint_quote.amount.into(),
        },
    )?;

    state
        .get_prices_config
        .write()
        .await
        .assets
        .insert(unit.matching_asset());

    Ok(())
}
