use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

use crate::error::CommandError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentDatabase {
    pub path: String,
    pub file_name: String,
    pub company_name: Option<String>,
    pub last_accessed: String,
}

/// Maximum number of recent databases to keep
const MAX_RECENT: usize = 20;

/// Get the path to the recent_databases.json file in the app config directory
fn get_recent_file_path(app: &AppHandle) -> Result<PathBuf, CommandError> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| CommandError::new(format!("Failed to get app config dir: {}", e)))?;
    Ok(config_dir.join("recent_databases.json"))
}

/// Load the recent databases list from disk
fn load_recent(app: &AppHandle) -> Result<Vec<RecentDatabase>, CommandError> {
    let file_path = get_recent_file_path(app)?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&file_path)
        .map_err(|e| CommandError::new(format!("Failed to read recent databases file: {}", e)))?;
    let list: Vec<RecentDatabase> = serde_json::from_str(&data)
        .map_err(|e| CommandError::new(format!("Failed to parse recent databases file: {}", e)))?;
    Ok(list)
}

/// Save the recent databases list to disk
fn save_recent(app: &AppHandle, list: &[RecentDatabase]) -> Result<(), CommandError> {
    let file_path = get_recent_file_path(app)?;
    // Ensure the parent directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CommandError::new(format!("Failed to create config directory: {}", e)))?;
    }
    let data = serde_json::to_string_pretty(list)
        .map_err(|e| CommandError::new(format!("Failed to serialize recent databases: {}", e)))?;
    fs::write(&file_path, data)
        .map_err(|e| CommandError::new(format!("Failed to write recent databases file: {}", e)))?;
    Ok(())
}

/// Get the list of recent databases, sorted by last_accessed descending
#[tauri::command]
pub fn get_recent_databases(app: AppHandle) -> Result<Vec<RecentDatabase>, CommandError> {
    let mut list = load_recent(&app)?;
    // Sort by last_accessed descending (most recent first)
    list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    Ok(list)
}

/// Add or update a database in the recent list
/// If the entry already exists, it is moved to the top with updated info
#[tauri::command]
pub fn add_recent_database(
    app: AppHandle,
    path: String,
    company_name: Option<String>,
) -> Result<Vec<RecentDatabase>, CommandError> {
    let mut list = load_recent(&app)?;

    // Extract file name from path
    let file_name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());

    // Remove existing entry if present
    list.retain(|entry| entry.path != path);

    // Add new entry at the beginning
    let entry = RecentDatabase {
        path,
        file_name,
        company_name,
        last_accessed: chrono::Utc::now().to_rfc3339(),
    };
    list.insert(0, entry);

    // Trim to MAX_RECENT
    list.truncate(MAX_RECENT);

    save_recent(&app, &list)?;

    // Return updated list sorted
    list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    Ok(list)
}

/// Remove a database from the recent list (does NOT delete the actual file)
#[tauri::command]
pub fn remove_recent_database(
    app: AppHandle,
    path: String,
) -> Result<Vec<RecentDatabase>, CommandError> {
    let mut list = load_recent(&app)?;
    list.retain(|entry| entry.path != path);
    save_recent(&app, &list)?;

    // Return updated list sorted
    list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    Ok(list)
}

/// Update the company name for an existing entry in the recent list
#[tauri::command]
pub fn update_recent_database_company(
    app: AppHandle,
    path: String,
    company_name: Option<String>,
) -> Result<Vec<RecentDatabase>, CommandError> {
    let mut list = load_recent(&app)?;

    if let Some(entry) = list.iter_mut().find(|e| e.path == path) {
        entry.company_name = company_name;
        entry.last_accessed = chrono::Utc::now().to_rfc3339();
    }

    save_recent(&app, &list)?;

    // Return updated list sorted
    list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
    Ok(list)
}
