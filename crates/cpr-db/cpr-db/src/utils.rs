use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Convert Decimal to i64 basis points (e.g., Decimal(0.04) -> 400, Decimal(1.5) -> 15000)
pub fn decimal_to_basis_points(dec: &Decimal) -> i64 {
    (dec * Decimal::new(10000, 0)).to_i64().unwrap_or(0)
}

/// Convert i64 basis points to Decimal (e.g., 400 -> Decimal(0.04), 15000 -> Decimal(1.5))
pub fn basis_points_to_decimal(bp: i64) -> Decimal {
    Decimal::new(bp, 4)
}

/// Convert Decimal hours to i64 thousandths (e.g., Decimal(40.5) -> 40500)
pub fn decimal_to_thousandths(dec: &Decimal) -> i64 {
    (dec * Decimal::new(1000, 0)).to_i64().unwrap_or(0)
}

/// Convert i64 thousandths to Decimal hours (e.g., 40500 -> Decimal(40.5))
pub fn thousandths_to_decimal(th: i64) -> Decimal {
    Decimal::new(th, 3)
}
