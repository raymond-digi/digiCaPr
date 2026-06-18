use thiserror::Error;

#[derive(Error, Debug)]
pub enum PayrollError {
    #[error("Invalid SIN: {0}")]
    InvalidSin(String),
    
    #[error("Invalid tax configuration for year {0}")]
    InvalidTaxConfig(i32),
    
    #[error("Invalid province: {0}")]
    InvalidProvince(String),
    
    #[error("Invalid pay amount: {0}")]
    InvalidPayAmount(String),
    
    #[error("Employee not found: {0}")]
    EmployeeNotFound(i64),
    
    #[error("Payroll record not found: {0}")]
    PayrollNotFound(i64),
    
    #[error("Invalid date range: {0}")]
    InvalidDateRange(String),
    
    #[error("Calculation error: {0}")]
    CalculationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, PayrollError>;
