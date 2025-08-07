use crate::AppState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    currency: String,
    value: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Price {
    symbol: String,
    price: Vec<Token>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponce {
    prices: Vec<Price>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrenciesResponce {
    currencies: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub async fn price_provider_add_assets(
    state: tauri::State<'_, AppState>,
    new_assets: Vec<String>,
) -> Result<(), Error> {
    let mut cfg = state.get_prices_config.write().await;
    cfg.assets.extend(new_assets);
    Ok(())
}

#[tauri::command]
pub async fn price_provider_add_currencies(
    state: tauri::State<'_, AppState>,
    new_currencies: Vec<String>,
) -> Result<(), Error> {
    let mut cfg = state.get_prices_config.write().await;
    cfg.currencies.extend(new_currencies);
    Ok(())
}

#[tauri::command]
pub async fn get_currencies(state: tauri::State<'_, AppState>,) -> Result<Vec<String>, Error> {
    let cfg = state.get_prices_config.read().await;
    let host = cfg.url.clone();
    let resp: CurrenciesResponce = reqwest::get(host + "/currencies")
        .await?
        .json::<CurrenciesResponce>()
        .await?;
    Ok(resp.currencies)
}
