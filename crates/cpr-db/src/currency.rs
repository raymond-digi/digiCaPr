use rusqlite::types::{Type, ValueRef};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CurrencyError {
    #[error("Negative currency value: {0}")]
    Negative(Decimal),
    #[error("Value too large for i64 cents representation: {0}")]
    Overflow(Decimal),
    #[error("Invalid currency scale (must have 2 or fewer decimal places)")]
    InvalidScale,
}

/// Helper to read a numeric value from SQLite that might be stored as INTEGER or REAL
/// Converts to i64 (cents) by rounding if necessary
pub fn read_numeric_as_i64(row: &rusqlite::Row, idx: usize) -> Result<i64, rusqlite::Error> {
    let value_ref = row.get_ref(idx)?;
    match value_ref {
        ValueRef::Integer(i) => Ok(i),
        ValueRef::Real(f) => {
            // In rusqlite, ValueRef::Real contains an f64 directly
            // Round to nearest integer (cents)
            Ok(f.round() as i64)
        }
        ValueRef::Null => Ok(0), // Default to 0 for NULL
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            idx,
            Type::Integer,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected numeric value")),
        )),
    }
}

/// Helper to read an optional hours value from SQLite that might be stored as INTEGER or REAL
/// Returns None for NULL, otherwise converts the numeric value to i64 (thousandths of hours)
pub fn read_optional_hours_as_i64(row: &rusqlite::Row, idx: usize) -> Result<Option<i64>, rusqlite::Error> {
    let value_ref = row.get_ref(idx)?;
    match value_ref {
        ValueRef::Integer(i) => Ok(Some(i)),
        ValueRef::Real(f) => {
            // Real value (e.g., from a previous version that stored hours as float)
            // Round to nearest integer (thousandths)
            Ok(Some(f.round() as i64))
        }
        ValueRef::Null => Ok(None),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            idx,
            Type::Integer,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected numeric value for hours")),
        )),
    }
}

pub fn validate_currency(value: &Decimal) -> Result<(), CurrencyError> {
    if *value < Decimal::ZERO {
        return Err(CurrencyError::Negative(*value));
    }
    // Ensure reasonable bounds for pay rates (e.g., $0.01 to $10M/year for salary or extreme hourly)
    if *value > dec!(0.0) && (*value < dec!(0.01) || *value > dec!(10000000.00)) {
        return Err(CurrencyError::Overflow(*value));
    }
    if value.scale() > 2 {
        return Err(CurrencyError::InvalidScale);
    }
    Ok(())
}

pub fn decimal_to_cents(value: &Decimal) -> Result<i64, CurrencyError> {
    let rounded = value.round_dp(2);
    validate_currency(&rounded)?;
    let cents = (rounded * Decimal::new(100, 0)).round_dp(0);
    cents.to_i64().ok_or_else(|| CurrencyError::Overflow(rounded))
}

/// Convert a Decimal to cents (i64) allowing negative values.
/// Used for vacation accrual amounts where payouts are stored as negative cents.
pub fn decimal_to_cents_signed(value: &Decimal) -> Result<i64, CurrencyError> {
    let rounded = value.round_dp(2);
    if rounded.scale() > 2 {
        return Err(CurrencyError::InvalidScale);
    }
    let cents = (rounded * Decimal::new(100, 0)).round_dp(0);
    cents.to_i64().ok_or_else(|| CurrencyError::Overflow(rounded))
}

pub fn cents_to_decimal(cents: i64) -> Result<Decimal, CurrencyError> {
    if cents < 0 {
        return Err(CurrencyError::Negative(Decimal::new(cents as i64, 2)));
    }
    Ok(Decimal::new(cents, 2))
}

/// Convert cents i64 to Decimal allowing negative values (e.g., vacation balance)
pub fn cents_to_decimal_signed(cents: i64) -> Result<Decimal, CurrencyError> {
    Ok(Decimal::new(cents, 2))
}

pub fn format_currency(value: &Decimal) -> String {
    format!("${:.2}", value)
}

/// Convert cents (i64) to Decimal with a descriptive error on failure.
/// Use this in row-closure contexts where `.map_err(|_| rusqlite::Error::InvalidQuery)` would lose context.
/// The `field` parameter identifies the database field for the error message.
pub fn convert_cents(cents: i64, field: &str) -> Result<Decimal, rusqlite::Error> {
    cents_to_decimal(cents).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Data error: '{}' has an invalid value — {} (stored as {} cents)", field, e, cents),
            )),
        )
    })
}

/// Convert cents (i64) to Decimal allowing negative values, with a descriptive error on failure.
/// The `field` parameter identifies the database field for the error message.
pub fn convert_cents_signed(cents: i64, field: &str) -> Result<Decimal, rusqlite::Error> {
    cents_to_decimal_signed(cents).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Data error: '{}' has an invalid value — {} (stored as {} cents)", field, e, cents),
            )),
        )
    })
}

/// Drop-in replacement for `cents_to_decimal(X).map_err(|_| rusqlite::Error::InvalidQuery)`.
/// Returns a descriptive error with the raw cents value when conversion fails.
pub fn try_cents(cents: i64) -> Result<Decimal, rusqlite::Error> {
    cents_to_decimal(cents).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Data error: {} (stored as {} cents)", e, cents),
            )),
        )
    })
}
