use std::{sync::Arc, time::Duration};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::{PriceConfig, PriceResponce};

async fn fetch_and_emit_prices(
    url: &str,
    app_thread: &tauri::AppHandle,
) -> Result<(), reqwest::Error> {
    let resp = reqwest::get(url).await?;
    let body = resp.json::<PriceResponce>().await?;
    let _ = app_thread.emit("new-price", body);
    Ok(())
}

pub async fn start_price_fetcher(
    config: Arc<RwLock<PriceConfig>>,
    host: String,
    app_thread: tauri::AppHandle,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let cfg = config.read().await;

        let mut currencies: Vec<_> = cfg.currencies.iter().cloned().collect();
        currencies.sort();
        let mut url = format!("{}/prices?currencies={}", host, currencies.join(","));

        if !cfg.assets.is_empty() {
            let mut assets_vec: Vec<_> = cfg.assets.iter().cloned().collect();
            assets_vec.sort();
            url.push_str("&assets=");
            url.push_str(&assets_vec.join(","));
        }

        if let Err(err) = fetch_and_emit_prices(&url, &app_thread).await {
            tracing::error!("price fetch error: {}", err);
        }
    }
}
