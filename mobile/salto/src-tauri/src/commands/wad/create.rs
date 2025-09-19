use std::str::FromStr;

use starknet_types::{Asset, AssetFromStrError, AssetToUnitConversionError};
use tauri::{AppHandle, State};
use tracing::{Level, event};

use crate::{AppState, front_events::emit_trigger_balance_poll};
use parse_asset_amount::{ParseAmountStringError, parse_asset_amount};

#[derive(Debug, thiserror::Error)]
pub enum CreateWadsError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::errors::Error),
    #[error(transparent)]
    Asset(#[from] AssetFromStrError),
    #[error("invalid amount: {0}")]
    Amount(#[from] ParseAmountStringError),
    #[error(transparent)]
    AssetToUnitConversion(#[from] AssetToUnitConversionError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error("not enought funds in node {0}")]
    NotEnoughFundsInNode(u32),
    #[error("failed to connect to node: {0}")]
    ConnectToNode(#[from] wallet::ConnectToNodeError),
    #[error("cached connection error: {0}")]
    CachedConnection(#[from] crate::connection_cache::ConnectionCacheError),
    #[error("failed to plan spending: {0}")]
    PlanSpending(#[from] wallet::send::PlanSpendingError),
    #[error("failed to load proofs to create wads: {0}")]
    LoadProofsAndCreateWads(#[from] wallet::send::LoadProofsAndCreateWadsError),
}

impl serde::Serialize for CreateWadsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
#[tracing::instrument(skip(app, state))]
pub async fn create_wads(
    app: AppHandle,
    state: State<'_, AppState>,
    amount: String,
    asset: String,
) -> Result<String, CreateWadsError> {
    let asset = Asset::from_str(&asset)?;
    let unit = asset.find_best_unit();
    let amount = parse_asset_amount(&amount, asset, unit)?;

    event!(name: "planning_wad_spending", Level::INFO,
        asset = %asset,
        unit = %unit,
        amount = %amount,
        "Planning wad spending"
    );

    let mut db_conn = state.pool.get()?;
    let amount_to_use_per_node = wallet::send::plan_spending(&db_conn, amount, unit, &[])?;

    event!(name: "spending_plan_created", Level::INFO,
        num_nodes = amount_to_use_per_node.len(),
        total_amount = %amount,
        "Spending plan created"
    );

    let mut node_and_proofs = Vec::with_capacity(amount_to_use_per_node.len());

    for (node_id, amount_to_use) in amount_to_use_per_node {
        let mut node_client = state
            .get_node_client_connection(node_id)
            .await
            .map_err(CreateWadsError::CachedConnection)?;

        let proofs_ids = wallet::fetch_inputs_ids_from_db_or_node(
            crate::SEED_PHRASE_MANAGER,
            state.pool.clone(),
            &mut node_client,
            node_id,
            amount_to_use,
            unit.as_str(),
        )
        .await?
        .ok_or(CreateWadsError::NotEnoughFundsInNode(node_id))?;

        // Get node URL for wad creation (still needed by the wallet library)
        let node_url = wallet::db::node::get_url_by_id(&db_conn, node_id)?
            .expect("ids come form DB, there should be an url");

        node_and_proofs.push(((node_id, node_url), proofs_ids));
    }

    event!(name: "creating_wads_from_proofs", Level::INFO,
        num_nodes = node_and_proofs.len(),
        unit = %unit,
        "Creating wads from proofs"
    );

    let wads = wallet::send::load_proofs_and_create_wads(
        &mut db_conn,
        node_and_proofs,
        unit.as_str(),
        None,
    )?;

    event!(name: "wads_created_successfully", Level::INFO,
        wad_string_length = wads.to_string().len(),
        unit = %unit,
        total_amount = %amount,
        "Wad created successfully"
    );

    // Trigger immediate balance polling
    let _ = emit_trigger_balance_poll(&app);

    Ok(wads.to_string())
}
