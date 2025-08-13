use std::str::FromStr;

use nuts::nut04::MintQuoteState;
use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError, STARKNET_STR};
use tauri::{AppHandle, Emitter, State};

use crate::{
    AppState,
    commands::BalanceChange,
    parse_asset_amount::{ParseAmountStringError, parse_asset_amount},
};

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
    let mut node_client = wallet::connect_to_node(&node_url).await?;

    let response = wallet::mint::create_quote(
        state.pool.clone(),
        &mut node_client,
        node_id,
        STARKNET_STR.to_string(),
        amount,
        unit,
    )
    .await?;

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
    let mut node_client = wallet::connect_to_node(&node_url).await?;

    let mint_quote = {
        let db_conn = state.pool.get()?;
        wallet::db::mint_quote::get(&db_conn, node_id, &quote_id)?
    };

    if mint_quote.state != MintQuoteState::Paid {
        return Err(RedeemQuoteError::QuoteNotPaid);
    }

    wallet::mint::redeem_quote(
        state.pool.clone(),
        &mut node_client,
        STARKNET_STR.to_string(),
        mint_quote.id,
        node_id,
        &mint_quote.unit,
        mint_quote.amount,
    )
    .await?;

    app.emit(
        "balance-increase",
        BalanceChange {
            node_id,
            unit: mint_quote.unit.as_str().to_string(),
            amount: mint_quote.amount.into(),
        },
    )?;

    Ok(())
}
