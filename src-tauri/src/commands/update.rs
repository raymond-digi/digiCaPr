use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::error::CommandError;

/// Information about an available update
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    pub download_url: String,
}

/// Check if a newer version is available via the Tauri updater
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<UpdateInfo>, CommandError> {
    let updater = app.updater().map_err(|e| CommandError::new(format!("Updater not available: {}", e)))?;

    match updater.check().await.map_err(|e| CommandError::new(format!("Update check failed: {}", e)))? {
        Some(update) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            notes: update.body.clone(),
            download_url: update.download_url.to_string(),
        })),
        None => Ok(None),
    }
}

/// Download and install an available update, then restart the app
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), CommandError> {
    let updater = app.updater().map_err(|e| CommandError::new(format!("Updater not available: {}", e)))?;

    let update = updater
        .check()
        .await
        .map_err(|e| CommandError::new(format!("Update check failed: {}", e)))?
        .ok_or_else(|| CommandError::new(String::from("No update available")))?;

    update
        .download_and_install(|_chunk_len, _content_len| {}, || {})
        .await
        .map_err(|e| CommandError::new(format!("Update install failed: {}", e)))?;

    app.restart();
}

/// Check if a newer tax config is available on GitHub Releases
/// Returns the newer year if an update is available, None otherwise
#[tauri::command]
pub async fn check_config_updates(year: i32) -> Result<Option<i32>, CommandError> {
    cpr_core::tax::config::check_github_update(year)
        .await
        .map_err(|e| CommandError::new(format!("Config update check failed: {}", e)))
}

/// Download and save an updated tax config from GitHub Releases
/// Returns the downloaded config as a JSON value
#[tauri::command]
pub async fn download_config_update(year: i32) -> Result<serde_json::Value, CommandError> {
    let config = cpr_core::tax::config::download_github_config(year)
        .await
        .map_err(|e| CommandError::new(format!("Config download failed: {}", e)))?;

    serde_json::to_value(&config).map_err(|e| CommandError::new(format!("Config serialization failed: {}", e)))
}
