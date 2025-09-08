use crate::{
    AppState,
    background_tasks::fetch_and_emit_prices,
    front_events::price_events::{OutOfSyncPriceEvent, emit_out_of_sync_price_event},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrenciesResponce {
    currencies: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
#[repr(transparent)]
#[error(transparent)]
pub struct SetPriceProviderCurrencyError(crate::background_tasks::Error);

impl serde::Serialize for SetPriceProviderCurrencyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn set_price_provider_currency(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    new_currency: String,
) -> Result<(), SetPriceProviderCurrencyError> {
    {
        let mut config = state.get_prices_config.write().await;
        config.currency = new_currency;
        config.status = Default::default();
    }

    let res = fetch_and_emit_prices(&app, &state.get_prices_config).await;
    if let Err(err) = res {
        tracing::error!("price fetch error: {}", err);
        emit_out_of_sync_price_event(&app, OutOfSyncPriceEvent)
            .map_err(|e| SetPriceProviderCurrencyError(crate::background_tasks::Error::Tauri(e)))?;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum GetCurrenciesError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl serde::Serialize for GetCurrenciesError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn get_currencies(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, GetCurrenciesError> {
    let cfg = state.get_prices_config.read().await;
    let host = cfg.url.clone();
    let resp: CurrenciesResponce = reqwest::get(host + "/currencies")
        .await?
        .json::<CurrenciesResponce>()
        .await?;
    Ok(resp.currencies)
}
