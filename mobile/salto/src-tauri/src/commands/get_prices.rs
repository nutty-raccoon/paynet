use tauri::{ async_runtime, State, AppHandle, Emitter };
use std::time::Duration;
use std::sync::PoisonError;
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::AppState;
use super::{PriceResponce, CurrenciesResponce};

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
pub async fn get_prices(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), Error> {
    let mut started = state.get_prices_starded.lock().map_err(|e| Error::Lock(e.to_string()))?;
    if *started {
        return Ok(());
    }
    *started = true;
    drop(started);

    async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            match reqwest::get("http://127.0.0.1:3007/prices").await {
                Ok(resp) => match resp.json::<PriceResponce>().await  {
                    Ok(body) => {
                        let _ = app.emit("new-price", body);
                    }
                    Err(err) => {
                        eprintln!("Erreur de parsing JSON: {}", err);
                    }
                },
                Err(err) => {
                    eprintln!("Erreur requête API: {}", err);
                }
            }
        }
    });
  Ok(())
}

#[tauri::command]
pub async fn get_currencies() -> Result<Vec<String>, Error>{
    let resp: CurrenciesResponce = reqwest::get("http://127.0.0.1:3007/currencies").await?.json::<CurrenciesResponce>().await?;
    Ok(resp.currencies)
}