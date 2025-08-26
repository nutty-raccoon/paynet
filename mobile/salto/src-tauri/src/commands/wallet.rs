use tauri::{AppHandle, State};
use wallet::seed_phrase;

use crate::AppState;

#[derive(Debug, thiserror::Error)]
pub enum InitWalletError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::wallet::Error),
}

impl serde::Serialize for InitWalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RestoreWalletError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    SeedPhrase(#[from] seed_phrase::Error),
    #[error(transparent)]
    Wallet(#[from] wallet::wallet::Error),
}

impl serde::Serialize for RestoreWalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CheckWalletError {
    #[error(transparent)]
    R2D2(#[from] r2d2::Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
}

impl serde::Serialize for CheckWalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitWalletResponse {
    seed_phrase: String,
}

#[tauri::command]
pub fn init_wallet(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<InitWalletResponse, InitWalletError> {
    let db_conn = state.pool.get()?;

    let seed_phrase = seed_phrase::create_random()?;
    wallet::wallet::init(&app.config().identifier, &db_conn, &seed_phrase)?;

    Ok(InitWalletResponse {
        seed_phrase: seed_phrase.to_string(),
    })
}

#[tauri::command]
pub fn restore_wallet(
    app: AppHandle,
    state: State<'_, AppState>,
    seed_phrase: String,
) -> Result<(), RestoreWalletError> {
    let db_conn = state.pool.get()?;

    let seed_phrase = seed_phrase::create_from_str(&seed_phrase)?;
    wallet::wallet::restore(&app.config().identifier, &db_conn, seed_phrase)?;

    Ok(())
}

#[tauri::command]
pub fn check_wallet_exists(state: State<'_, AppState>) -> Result<bool, CheckWalletError> {
    let db_conn = state.pool.get()?;

    let wallet_count = wallet::db::wallet::count_wallets(&db_conn)?;
    Ok(wallet_count > 0)
}
