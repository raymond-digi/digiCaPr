pub mod schema;
pub mod repository;
pub mod error;
pub mod password;
pub mod currency;
pub mod utils;

pub use error::{DbError, DbResult};
pub use password::{generate_hash, verify_hash};
pub use repository::{Database, EmployeeRepository, PayrollHistoryRepository, CompanyRepository, VacationRepository, ConfigRepository, PayrollRepository};
pub use currency::{decimal_to_cents, cents_to_decimal, validate_currency};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;


/// Initialize a new payroll database
pub fn init_database<P: AsRef<Path>>(path: P) -> DbResult<Connection> {
    let mut conn = Connection::open(path)?;
    schema::initialize_database(&mut conn)?;
    Ok(conn)
}

/// Open an existing payroll database
pub fn open_database<P: AsRef<Path>>(path: P, password: Option<&str>) -> DbResult<Connection> {
    let mut conn = Connection::open(path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Initialize schema from embedded full schema SQL (idempotent)
    schema::initialize_database(&mut conn)?;

    // Check if password is required
    let config_row = conn.query_row(
        "SELECT is_locked, password_hash FROM config WHERE id = 1",
        [],
        |row| Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, Option<String>>(1)?,
        )),
    ).optional()?;

    match config_row {
        Some((1, Some(hash))) => {
            let provided = password.ok_or_else(|| DbError::InvalidData("Database is password protected. Password required.".to_string()))?;
            if !verify_hash(&hash, provided)? {
                return Err(DbError::InvalidData("Invalid password.".to_string()));
            }
        }
        Some((1, None)) => {
            return Err(DbError::InvalidData("Database is locked but no password hash configured.".to_string()));
        }
        _ => {
            // Unlocked database or no config - optional password ignored
            if password.is_some() {
                // Could log warning here in future
            }
        }
    }

    Ok(conn)
}
