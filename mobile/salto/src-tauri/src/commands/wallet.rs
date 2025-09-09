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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitWalletResponse {
    seed_phrase: String,
}

#[tauri::command]
pub fn init_wallet(state: State<'_, AppState>) -> Result<InitWalletResponse, InitWalletError> {
    let db_conn = state.pool.get()?;

    let seed_phrase = seed_phrase::create_random()?;
    wallet::wallet::init(crate::SEED_PHRASE_MANAGER, &db_conn, &seed_phrase)?;

    Ok(InitWalletResponse {
        seed_phrase: seed_phrase.to_string(),
    })
}

#[tauri::command]
pub fn restore_wallet(
    state: State<'_, AppState>,
    seed_phrase: String,
) -> Result<(), RestoreWalletError> {
    let db_conn = state.pool.get()?;

    let seed_phrase = seed_phrase::create_from_str(&seed_phrase)?;
    let _opt_prev_seed_phrase =
        wallet::wallet::restore(crate::SEED_PHRASE_MANAGER, &db_conn, seed_phrase)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CheckWalletError {
    #[error(transparent)]
    Wallet(#[from] wallet::wallet::Error),
}

impl serde::Serialize for CheckWalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
pub fn check_wallet_exists() -> Result<bool, CheckWalletError> {
    let exists = wallet::wallet::exists(crate::SEED_PHRASE_MANAGER)?;

    Ok(exists)
}

#[derive(Debug, thiserror::Error)]
pub enum GetSeedPhraseError {
    #[error(transparent)]
    Wallet(#[from] wallet::wallet::Error),
    #[cfg(any(target_os = "android", target_os = "ios"))]
    #[error(transparent)]
    Biometric(#[from] tauri_plugin_biometric::Error),
}

impl serde::Serialize for GetSeedPhraseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[tauri::command]
#[allow(unused_variables)]
pub fn get_seed_phrase(app: AppHandle) -> Result<String, GetSeedPhraseError> {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    bio_auth(&app)?;

    let mnemonic = wallet::wallet::get_seed_phrase(crate::SEED_PHRASE_MANAGER)?;

    Ok(mnemonic.to_string())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn bio_auth(app: &AppHandle) -> tauri_plugin_biometric::Result<()> {
    use tauri_plugin_biometric::AuthOptions;
    use tauri_plugin_biometric::BiometricExt;

    let options = AuthOptions {
        // Set True if you want the user to be able to authenticate using phone password
        allow_device_credential: false,
        cancel_title: None,

        // iOS only feature
        fallback_title: None,

        // Android only features
        title: Some("Reveal seed phrase".to_string()),
        subtitle: None,
        confirmation_required: None,
    };

    app.biometric()
        .authenticate("Unlock to show".to_string(), options)
}
