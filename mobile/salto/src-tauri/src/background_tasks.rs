use std::{sync::Arc, time::Duration};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::PriceConfig;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NewPriceResp {
    symbol: String,
    value: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PriceProviderResponse {
    prices: Vec<TokenPrice>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TokenPrice {
    symbol: String,
    price: Vec<CurrencyValue>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurrencyValue {
    currency: String,
    value: f64,
}

fn pick_value(tokens: &[CurrencyValue], wanted: &str) -> Option<f64> {
    tokens
        .iter()
        .find(|t| t.currency.eq_ignore_ascii_case(wanted))
        .map(|t| t.value)
        .or_else(|| tokens.first().map(|t| t.value))
}

async fn fetch_and_emit_prices(
    url: &str,
    app: &tauri::AppHandle,
    config: &Arc<RwLock<PriceConfig>>,
) -> Result<(), Error> {
    let resp: PriceProviderResponse = reqwest::get(url).await?.error_for_status()?.json().await?;
    let currency = &config.read().await.currency;
    let payload: Vec<NewPriceResp> = resp
        .prices
        .into_iter()
        .filter_map(|p| {
            pick_value(&p.price, currency).map(|v| NewPriceResp {
                symbol: p.symbol,
                value: v,
            })
        })
        .collect();
    app.emit("new-price", payload)?;
    Ok(())
}

pub async fn start_price_fetcher(config: Arc<RwLock<PriceConfig>>, app_thread: tauri::AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let (host, currency, assets) = {
            let cfg = config.read().await;
            (cfg.url.clone(), cfg.currency.clone(), {
                let mut a: Vec<_> = cfg.assets.iter().cloned().collect();
                a.sort();
                a
            })
        };
        let mut url = format!("{}/prices?currencies={}", host, currency);
        url.push_str("&assets=");
        url.push_str(&assets.join(","));

        if let Err(err) = fetch_and_emit_prices(&url, &app_thread, &config).await {
            tracing::error!("price fetch error: {}", err);
        }
    }
}
