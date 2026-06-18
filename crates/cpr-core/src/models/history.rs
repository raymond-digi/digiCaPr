use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::PayType;

/// Pay rate history entry - tracks changes in employee compensation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayRateHistory {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub pay_rate: Decimal,
    pub pay_type: PayType,
    pub effective_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl PayRateHistory {
    pub fn new(
        employee_id: i64,
        pay_rate: Decimal,
        pay_type: PayType,
        effective_date: NaiveDate,
        reason: Option<String>,
    ) -> Self {
        Self {
            id: None,
            employee_id,
            pay_rate,
            pay_type,
            effective_date,
            end_date: None,
            reason,
            created_at: Utc::now(),
        }
    }
    
    /// Check if this pay rate is currently active
    pub fn is_active(&self) -> bool {
        self.end_date.is_none()
    }
    
    /// Check if this pay rate was active on a specific date
    pub fn is_active_on(&self, date: &NaiveDate) -> bool {
        *date >= self.effective_date && 
        self.end_date.map_or(true, |end| *date <= end)
    }
}

/// Employment history entry - tracks periods of employment for rehires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentHistory {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub termination_reason: Option<String>,
    pub rehire_eligible: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl EmploymentHistory {
    pub fn new(
        employee_id: i64,
        hire_date: NaiveDate,
    ) -> Self {
        Self {
            id: None,
            employee_id,
            hire_date,
            termination_date: None,
            termination_reason: None,
            rehire_eligible: true,
            notes: None,
            created_at: Utc::now(),
        }
    }
    
    /// Check if this is the current employment period
    pub fn is_current(&self) -> bool {
        self.termination_date.is_none()
    }
    
    /// Terminate this employment period
    pub fn terminate(&mut self, date: NaiveDate, reason: Option<String>, eligible_for_rehire: bool) {
        self.termination_date = Some(date);
        self.termination_reason = reason;
        self.rehire_eligible = eligible_for_rehire;
    }
    
    /// Calculate duration in days
    pub fn duration_days(&self) -> i64 {
        let end = self.termination_date.unwrap_or_else(|| Utc::now().date_naive());
        (end - self.hire_date).num_days()
    }
}
