use itertools::intersperse;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::error;

use crate::{
    AppState, PriceConfig, PriceSyncStatus,
    front_events::{
        NewPriceEvent, OutOfSyncPriceEvent, emit_new_price_event, emit_out_of_sync_price_event,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
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

pub async fn fetch_and_emit_prices(
    app: &tauri::AppHandle,
    config: &Arc<RwLock<PriceConfig>>,
) -> Result<(), Error> {
    let url = {
        let cfg = config.read().await;
        let mut url = format!("{}/prices?currencies={}&assets=", cfg.url, cfg.currency);
        url.extend(intersperse(cfg.assets.iter().map(|a| a.as_str()), ","));
        url
    };
    let resp: PriceProviderResponse = reqwest::get(url).await?.error_for_status()?.json().await?;

    let payload: Vec<NewPriceEvent> = {
        let currency = &config.read().await.currency;
        resp.prices
            .into_iter()
            .filter_map(|p| {
                pick_value(&p.price, currency).map(|v| NewPriceEvent {
                    symbol: p.symbol,
                    value: v,
                })
            })
            .collect()
    };

    emit_new_price_event(app, payload)?;
    config.write().await.status = PriceSyncStatus::Synced(SystemTime::now());

    Ok(())
}

// TODO: pause price fetching when app is not used (background/not-focused)
pub async fn start_price_fetcher(app: tauri::AppHandle) {
    let config = app.state::<AppState>().get_prices_config.clone();
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
                    if let Err(e) = emit_out_of_sync_price_event(&app, OutOfSyncPriceEvent) {
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
