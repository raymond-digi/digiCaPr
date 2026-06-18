use tauri::State;
use serde::{Deserialize, Serialize};

use cpr_core::models::{RegistryEntry, RegistryValue};
use crate::state::AppState;
use crate::error::CommandError;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrySetRequest {
    pub key_path: String,
    pub value_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryGetRequest {
    pub key_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryDeleteRequest {
    pub key_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryListRequest {
    pub path_prefix: String,
}

/// Helper to get database from state
fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&cpr_db::repository::Database) -> Result<T, CommandError>,
{
    let db_guard = state.database.lock().unwrap();
    let db = db_guard
        .as_ref()
        .ok_or_else(|| CommandError::new("No database open"))?;
    f(db)
}

#[tauri::command]
pub fn registry_set(
    state: State<'_, AppState>,
    request: RegistrySetRequest,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        let value = RegistryValue::from_string_and_type(&request.value, &request.value_type)
            .map_err(|e| CommandError::new(&e.to_string()))?;
        registry
            .set(&request.key_path, &value)
            .map_err(|e| CommandError::new(&e.to_string()))?;
        Ok(())
    })
}

#[tauri::command]
pub fn registry_get(
    state: State<'_, AppState>,
    request: RegistryGetRequest,
) -> Result<Option<RegistryEntry>, CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .get(&request.key_path)
            .map_err(|e| CommandError::new(&e.to_string()))
    })
}

#[tauri::command]
pub fn registry_delete(
    state: State<'_, AppState>,
    request: RegistryDeleteRequest,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .delete(&request.key_path)
            .map_err(|e| CommandError::new(&e.to_string()))?;
        Ok(())
    })
}

#[tauri::command]
pub fn registry_exists(
    state: State<'_, AppState>,
    request: RegistryGetRequest,
) -> Result<bool, CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .exists(&request.key_path)
            .map_err(|e| CommandError::new(&e.to_string()))
    })
}

#[tauri::command]
pub fn registry_list_keys(
    state: State<'_, AppState>,
    request: RegistryListRequest,
) -> Result<Vec<String>, CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .list_keys(&request.path_prefix)
            .map_err(|e| CommandError::new(&e.to_string()))
    })
}

#[tauri::command]
pub fn registry_get_all(
    state: State<'_, AppState>,
    request: RegistryListRequest,
) -> Result<Vec<RegistryEntry>, CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .get_all(&request.path_prefix)
            .map_err(|e| CommandError::new(&e.to_string()))
    })
}

#[tauri::command]
pub fn registry_delete_all(
    state: State<'_, AppState>,
    request: RegistryListRequest,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let registry = db.registry();
        registry
            .delete_all(&request.path_prefix)
            .map_err(|e| CommandError::new(&e.to_string()))?;
        Ok(())
    })
}
