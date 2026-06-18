use tauri::State;
use crate::state::AppState;
use crate::error::CommandError;
use cpr_core::models::Company;

/// Helper to get database from state
fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&cpr_db::repository::Database) -> Result<T, CommandError>,
{
    let db_guard = state.database.lock().unwrap();
    let db = db_guard.as_ref()
        .ok_or_else(|| CommandError::new("No database open"))?;
    f(db)
}

/// Get company information
#[tauri::command]
pub async fn get_company(
    state: State<'_, AppState>,
) -> Result<Option<Company>, CommandError> {
    with_db(&state, |db| {
        let company = db.company().get()?;
        Ok(company)
    })
}

/// Save company information (create or update)
#[tauri::command]
pub async fn save_company(
    mut company: Company,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    company.id = Some(1);
    with_db(&state, |db| {
        db.company().upsert(&company)?;
        Ok(())
    })
}
