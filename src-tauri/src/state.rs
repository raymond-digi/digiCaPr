use std::sync::Mutex;
use cpr_db::repository::Database;

/// Application state managed by Tauri
/// Holds the database connection and path
pub struct AppState {
    /// Current database file path
    pub db_path: Mutex<Option<String>>,
    /// Active database wrapper
    pub database: Mutex<Option<Database>>,
}

impl AppState {
    /// Create new application state with no active database
    pub fn new() -> Self {
        Self {
            db_path: Mutex::new(None),
            database: Mutex::new(None),
        }
    }
    
    /// Set the database connection and path
    pub fn set_database(&self, path: String, db: Database) {
        *self.db_path.lock().unwrap() = Some(path);
        *self.database.lock().unwrap() = Some(db);
    }
    
    /// Clear the database connection
    pub fn clear_database(&self) {
        *self.db_path.lock().unwrap() = None;
        *self.database.lock().unwrap() = None;
    }
    
    /// Get the current database path
    pub fn get_db_path(&self) -> Option<String> {
        self.db_path.lock().unwrap().clone()
    }
    
    /// Check if a database is currently open
    pub fn has_database(&self) -> bool {
        self.database.lock().unwrap().is_some()
    }
    
    /// Get the current database (for commands that require open DB)
    #[allow(dead_code)]
    pub fn get_database(&self) -> Result<std::sync::MutexGuard<'_, Option<cpr_db::repository::Database>>, crate::error::CommandError> {
        Ok(self.database.lock().unwrap())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
