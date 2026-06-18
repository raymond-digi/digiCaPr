use tauri::State;
use crate::state::AppState;
use crate::error::CommandError;
use cpr_db::repository::Database;

/// Create a new database at the specified path
#[tauri::command]
pub async fn create_database(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    // Create new database
    let conn = cpr_db::init_database(&path)?;
    let db = Database::new(conn);
    
    // Store in app state
    state.set_database(path.clone(), db);
    
    Ok(path)
}

/** Open an existing database with optional password */
#[tauri::command]
pub async fn open_database(
    path: String,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    // Open existing database
    let conn = cpr_db::open_database(&path, password.as_deref())?;
    let db = Database::new(conn);
    
    // Store in app state
    state.set_database(path.clone(), db);
    
    Ok(path)
}

/// Close the current database
#[tauri::command]
pub async fn close_database(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.clear_database();
    Ok(())
}

/// Get the current database path
#[tauri::command]
pub async fn get_current_database_path(
    state: State<'_, AppState>,
) -> Result<Option<String>, CommandError> {
    Ok(state.get_db_path())
}

/// Check if a database is currently open
#[tauri::command]
pub async fn is_database_open(state: State<'_, AppState>) -> Result<bool, CommandError> {
    Ok(state.has_database())
}

#[allow(dead_code)]
/// Get password hint for current database
#[tauri::command]
pub async fn get_password_hint(state: State<'_, AppState>) -> Result<Option<String>, CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or_else(|| CommandError::new("No database open"))?;
    db.config().get_hint().map_err(|e| CommandError::new(e.to_string()))
}

#[allow(dead_code)]
/// Set database password
#[tauri::command]
pub async fn set_database_password(
    password: String,
    hint: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    db.config().set_password(&password, hint.as_deref())?;
    Ok(())
}

#[allow(dead_code)]
/// Set recovery key
#[tauri::command]
pub async fn set_recovery_key(
    recovery_key: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    db.config().set_recovery_key(&recovery_key)?;
    Ok(())
}

#[allow(dead_code)]
/// Reset password using recovery key
#[tauri::command]
pub async fn reset_database_password(
    recovery_key: String,
    new_password: String,
    new_hint: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    db.config().reset_password(&recovery_key, &new_password, new_hint.as_deref())?;
    Ok(())
}

#[allow(dead_code)]
/** Remove database password protection */
#[tauri::command]
pub async fn remove_database_password(state: State<'_, AppState>) -> Result<(), CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    db.config().remove_password()?;
    Ok(())
}

#[allow(dead_code)]
/// Get master support email
#[tauri::command]
pub async fn get_support_email(state: State<'_, AppState>) -> Result<String, CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    Ok(db.config().get_support_email()?)
}

#[allow(dead_code)]
/// Set master support email (admin)
#[tauri::command]
pub async fn set_support_email(
    email: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    let db_guard = state.get_database()?;
    let db = db_guard.as_ref().ok_or(CommandError::new("No database open"))?;
    db.config().set_support_email(&email)?;
    Ok(())
}
