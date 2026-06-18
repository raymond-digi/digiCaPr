use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Transaction type for vacation accrual records
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VacationTransactionType {
    /// Automatic accrual from payroll
    Accrue,
    /// Vacation paid out (from payroll vacation earning)
    Payout,
    /// Manual adjustment (correction, policy, forfeited, etc.)
    Adjust,
    /// Unpaid time off taken (reduces balance without payout)
    Timeoff,
}

impl VacationTransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accrue => "accrue",
            Self::Payout => "payout",
            Self::Adjust => "adjust",
            Self::Timeoff => "timeoff",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "accrue" => Some(Self::Accrue),
            "payout" => Some(Self::Payout),
            "adjust" => Some(Self::Adjust),
            "timeoff" => Some(Self::Timeoff),
            _ => None,
        }
    }
}

/// Vacation accrual record - tracks all vacation balance changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacationAccrual {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub accrual_date: NaiveDate,
    pub payroll_id: Option<i64>,
    pub transaction_type: VacationTransactionType,
    /// Store as cents (+ to add, - to deduct) — dollar-based accrual
    pub amount: Decimal,
    /// Running dollar balance after this transaction (for audit)
    pub balance_after: Decimal,
    /// Day-based accrual amount (+ to add, - to deduct), stored as days (thousandths internally in DB)
    /// For hourly employees: derived from dollar amount / daily rate
    /// For non-hourly employees: primary value from vacation_pay_rate × 250 / pay_periods
    pub amount_days: Decimal,
    /// Running day balance after this transaction (for audit)
    pub balance_after_days: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl VacationAccrual {
    pub fn new(
        employee_id: i64,
        accrual_date: NaiveDate,
        transaction_type: VacationTransactionType,
        amount: Decimal,
        balance_after: Decimal,
    ) -> Self {
        Self {
            id: None,
            employee_id,
            accrual_date,
            payroll_id: None,
            transaction_type,
            amount,
            balance_after,
            amount_days: Decimal::ZERO,
            balance_after_days: Decimal::ZERO,
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new accrual with both dollar and day values
    pub fn new_dual(
        employee_id: i64,
        accrual_date: NaiveDate,
        transaction_type: VacationTransactionType,
        amount: Decimal,
        balance_after: Decimal,
        amount_days: Decimal,
        balance_after_days: Decimal,
    ) -> Self {
        Self {
            id: None,
            employee_id,
            accrual_date,
            payroll_id: None,
            transaction_type,
            amount,
            balance_after,
            amount_days,
            balance_after_days,
            notes: None,
            created_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.amount == Decimal::ZERO && self.amount_days == Decimal::ZERO {
            return Err(crate::PayrollError::ValidationError(
                "Transaction amount cannot be zero (both dollar and day amounts are zero)".to_string(),
            ));
        }
        Ok(())
    }
}

/// Vacation time off record - tracks vacation taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacationTimeOff {
    pub id: Option<i64>,
    pub employee_id: i64,
    /// Links to the accrual transaction if this is a paid time off
    pub vacation_accrual_id: Option<i64>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub pay_type: VacationPayType,
    /// Initial estimated payout amount (readonly reference, in cents)
    pub estimated_payout: Decimal,
    /// Editable payout amount (in cents)
    pub payout_amount: Decimal,
    /// Number of weekdays taken as vacation (for non-hourly employees)
    /// Stored as days (e.g., 1.5 = 1.5 days). None for hourly employees.
    pub days_taken: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Whether vacation time off is paid or unpaid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VacationPayType {
    Paid,
    Unpaid,
}

impl VacationPayType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Paid => "paid",
            Self::Unpaid => "unpaid",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "paid" => Some(Self::Paid),
            "unpaid" => Some(Self::Unpaid),
            _ => None,
        }
    }
}

impl VacationTimeOff {
    pub fn new(
        employee_id: i64,
        start_date: NaiveDate,
        end_date: NaiveDate,
        pay_type: VacationPayType,
        estimated_payout: Decimal,
        payout_amount: Decimal,
    ) -> Self {
        Self {
            id: None,
            employee_id,
            vacation_accrual_id: None,
            start_date,
            end_date,
            pay_type,
            estimated_payout,
            payout_amount,
            days_taken: None,
            notes: None,
            created_at: Utc::now(),
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.end_date < self.start_date {
            return Err(crate::PayrollError::ValidationError(
                "End date cannot be before start date".to_string(),
            ));
        }

        Ok(())
    }
}

/// Standard working days per year (260 total - 10 vacation base = 250)
/// Used to convert vacation_pay_rate (%) to vacation days for non-hourly employees.
/// Example: 4% × 250 = 10 days/year, 6% × 250 = 15 days/year
pub const WORK_DAYS_PER_YEAR: i32 = 250;

/// Calculate vacation pay accrual dollar amount based on employee's vacation pay rate
pub fn calculate_vacation_accrual(
    gross_pay: Decimal,
    vacation_pay_rate: Decimal,
) -> Decimal {
    if vacation_pay_rate <= Decimal::ZERO || gross_pay <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    gross_pay * vacation_pay_rate
}

/// Calculate vacation accrual in days for non-hourly employees.
/// Formula: vacation_pay_rate × WORK_DAYS_PER_YEAR / total_pay_periods
///
/// Example: 4% rate, 26 pay periods → 0.04 × 250 / 26 = 0.3846 days/period
/// Example: 6% rate, 26 pay periods → 0.06 × 250 / 26 = 0.5769 days/period
pub fn calculate_vacation_accrual_days(
    vacation_pay_rate: Decimal,
    total_pay_periods: i32,
) -> Decimal {
    if vacation_pay_rate <= Decimal::ZERO || total_pay_periods <= 0 {
        return Decimal::ZERO;
    }
    let work_days = Decimal::from(WORK_DAYS_PER_YEAR);
    let periods = Decimal::from(total_pay_periods);
    vacation_pay_rate * work_days / periods
}

/// Convert weekday count to a decimal days value
pub fn weekdays_to_days(weekdays: i64) -> Decimal {
    Decimal::from(weekdays)
}

/// Count weekdays (Mon-Fri) between two dates inclusive
pub fn count_weekdays(start_date: NaiveDate, end_date: NaiveDate) -> i64 {
    if end_date < start_date {
        return 0;
    }
    let mut count = 0i64;
    let mut current = start_date;
    while current <= end_date {
        match current.weekday() {
            chrono::Weekday::Mon | chrono::Weekday::Tue | chrono::Weekday::Wed
            | chrono::Weekday::Thu | chrono::Weekday::Fri => count += 1,
            _ => {}
        }
        current = current.succ_opt().unwrap_or(current);
    }
    count
}
