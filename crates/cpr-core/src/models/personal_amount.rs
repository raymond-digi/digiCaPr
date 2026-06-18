use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAmount {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub province: String,
    pub year: i32,
    pub federal_amount: Decimal,
    pub provincial_amount: Decimal,
    pub indexed_at: DateTime<Utc>,
}