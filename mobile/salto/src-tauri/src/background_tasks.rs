use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::{PriceConfig, PriceSyncStatus};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
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

fn format_balance(assets: Vec<String>) -> Vec<String> {
    assets
        .into_iter()
        .map(|a| match a.as_str() {
            "millistrk" => "strk".to_string(),
            "gwei" => "ETH".to_string(),
            _ => a,
        })
        .collect()
}

fn pick_value(tokens: &[CurrencyValue], wanted: &str) -> Option<f64> {
    tokens
        .iter()
        .find(|t| t.currency.eq_ignore_ascii_case(wanted))
        .map(|t| t.value)
        .or_else(|| tokens.first().map(|t| t.value))
}

pub async fn fetch_and_emit_prices(
    app: &tauri::AppHandle,
    config: &Arc<RwLock<PriceConfig>>,
) -> Result<(), Error> {
    let (host, currency, assets) = {
        let cfg = config.read().await;
        (cfg.url.clone(), cfg.currency.clone(), {
            let mut a: Vec<_> = cfg.assets.iter().cloned().collect();
            a.sort();
            a
        })
    };
    let format_assets = format_balance(assets);
    let mut url = format!("{}/prices?currencies={}", host, currency);
    url.push_str("&assets=");
    url.push_str(&format_assets.join(","));

    let resp: PriceProviderResponse = reqwest::get(url).await?.error_for_status()?.json().await?;

    let payload: Vec<NewPriceResp> = {
        let currency = &config.read().await.currency;
        resp.prices
            .into_iter()
            .filter_map(|p| {
                pick_value(&p.price, currency).map(|v| NewPriceResp {
                    symbol: p.symbol,
                    value: v,
                })
            })
            .collect()
    };

    app.emit("new-price", payload)?;
    config.write().await.status = PriceSyncStatus::Synced(SystemTime::now());

    Ok(())
}

// TODO: pause price fetching when app is not used (background/not-focused)
pub async fn start_price_fetcher(config: Arc<RwLock<PriceConfig>>, app: tauri::AppHandle) {
    let mut retry_delay = 1;
    loop {
        let res = fetch_and_emit_prices(&app, &config).await;
        if let Err(err) = res {
            tracing::error!("price fetch error: {}", err);
            match config.read().await.status {
                crate::PriceSyncStatus::Synced(last_sync_time)
                    if SystemTime::now()
                        .duration_since(last_sync_time)
                        .unwrap()
                        .as_secs()
                        > 60 =>
                {
                    if let Err(e) = app.emit("out-of-sync-price", ()) {
                        tracing::error!("failed to signal price out of sync: {e}");
                    }
                }
                _ => {}
            };

            tokio::time::sleep(Duration::from_secs(retry_delay)).await;
            retry_delay = std::cmp::min(60, retry_delay * 2);
        } else {
            retry_delay = 1;
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}
