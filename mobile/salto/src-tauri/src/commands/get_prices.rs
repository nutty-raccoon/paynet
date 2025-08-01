use std::{env, time::Duration};
use tauri::{AppHandle, Emitter, async_runtime};

use crate::AppState;
use crate::PriceConfig;

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
struct PriceResponce {
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
    #[error("mutex lock error: {0}")]
    Lock(String),
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
pub async fn update_get_prices_config(
    state: tauri::State<'_, AppState>,
    currencies: Vec<String>,
    new_assets: Option<Vec<String>>,
) -> Result<(), Error> {
    let assets = match new_assets {
        Some(_) => Some(new_assets.unwrap()),
        None => {
            let cfg = state.get_prices_config.write().await;
            cfg.assets.clone()
        }
    };
    let mut cfg = state.get_prices_config.write().await;
    *cfg = PriceConfig {
        currencies,
        assets: assets,
    };
    Ok(())
}

#[tauri::command]
pub async fn get_prices(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    currencies: Vec<String>,
    assets: Option<Vec<String>>,
) -> Result<(), Error> {
    let mut cfg = state.get_prices_config.write().await;
    let host = env::var("PRICE_PROVIDER").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let mut started = state
        .get_prices_starded
        .lock()
        .map_err(|e| Error::Lock(e.to_string()))?;
    if *started {
        return Ok(());
    }
    *started = true;

    drop(started);
    *cfg = PriceConfig { currencies, assets };
    let config = state.get_prices_config.clone();

    async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let cfg = config.read().await;
            let mut url = format!("{}/prices?currencies={}", host, cfg.currencies.join(","));
            if let Some(tokens) = &cfg.assets {
                if !tokens.is_empty() {
                    url.push_str("&assets=");
                    url.push_str(&tokens.join(","));
                }
            }
            println!("this is url: {}", url);
            match reqwest::get(url).await {
                Ok(resp) => match resp.json::<PriceResponce>().await {
                    Ok(body) => {
                        let _ = app.emit("new-price", body);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                },
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn get_currencies() -> Result<Vec<String>, Error> {
    let host = env::var("PRICE_PROVIDER").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let resp: CurrenciesResponce = reqwest::get(host + "/currencies")
        .await?
        .json::<CurrenciesResponce>()
        .await?;
    Ok(resp.currencies)
}
