use thiserror::Error;
use crate::currency::CurrencyError;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    
    #[error("Record not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Core error: {0}")]
    Core(#[from] cpr_core::PayrollError),

    #[error("Currency error: {0}")]
    Currency(#[from] CurrencyError),
    
    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Password error: {0}")]
    Password(String),

    #[error("Personal amount not found for employee {employee_id} in {province} for year {year}. Fallback to latest available amount.")]
    PersonalAmountNotFound { employee_id: i64, year: i32, province: String },

    #[error("No personal amounts available for employee {employee_id} in province {province}.")]
    NoPersonalAmountsAvailable { employee_id: i64, province: String },
}

pub type DbResult<T> = Result<T, DbError>;
