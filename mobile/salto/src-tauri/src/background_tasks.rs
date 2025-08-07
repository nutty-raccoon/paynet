use std::{sync::Arc, time::Duration};
use tauri::Emitter;
use tokio::sync::RwLock;

use crate::{PriceConfig, PriceResponce};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

async fn fetch_and_emit_prices(
    url: &str,
    app: &tauri::AppHandle,
) -> Result<(), Error> {
    let resp = reqwest::get(url).await?;
    let new_prices = resp.json::<PriceResponce>().await?;
    app.emit("new-price", new_prices)?;
    Ok(())
}

pub async fn start_price_fetcher(
    config: Arc<RwLock<PriceConfig>>,
    app_thread: tauri::AppHandle,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        let cfg = config.read().await;

        if !cfg.assets.is_empty() && !cfg.currencies.is_empty() {
            break;
        }

        drop(cfg);
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    loop {
        interval.tick().await;

        let (host, currencies, assets) = {
            let cfg = config.read().await;
            (
                cfg.url.clone(),
                {
                    let mut c: Vec<_> = cfg.currencies.iter().cloned().collect();
                    c.sort();
                    c
                },
                {
                    let mut a: Vec<_> = cfg.assets.iter().cloned().collect();
                    a.sort();
                    a
                },
            )
        };
        let mut url = format!("{}/prices?currencies={}", host, currencies.join(","));
        url.push_str("&assets=");
        url.push_str(&assets.join(","));

        if let Err(err) = fetch_and_emit_prices(&url, &app_thread).await {
            tracing::error!("price fetch error: {}", err);
        }
    }
}
