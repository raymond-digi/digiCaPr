use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payroll {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub pay_date: NaiveDate,
    pub regular_hours: Option<Decimal>,
    pub overtime_hours: Option<Decimal>,
    pub additional_earnings: Vec<AdditionalEarning>,
    pub insured_earning: Decimal,
    pub gross_pay: Decimal,
    pub additional_earnings_total: Decimal,
    pub additional_tax_amount: Decimal,
    pub deductions: Deductions,
    pub net_pay: Decimal,
    pub pay_period_number: Option<i32>,
    pub total_pay_periods: i32,
    pub total_deductions: Decimal,
    pub additional_deductions: Decimal,
    pub federal_personal_amount: Decimal,
    pub provincial_personal_amount: Decimal,
    pub province: String,
    pub remittance_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deductions {
    pub cpp: Decimal,
    pub cpp2: Decimal,
    pub ei: Decimal,
    pub federal_tax: Decimal,
    pub provincial_tax: Decimal,
    pub additional: Vec<AdditionalDeduction>,
}

impl Deductions {
    pub fn new() -> Self {
        Self { cpp: Decimal::ZERO, cpp2: Decimal::ZERO, ei: Decimal::ZERO, federal_tax: Decimal::ZERO, provincial_tax: Decimal::ZERO, additional: Vec::new() }
    }

    pub fn total(&self) -> Decimal {
        let additional_total: Decimal = self.additional.iter().map(|d| d.amount).sum();

        self.cpp + self.cpp2 + self.ei + self.federal_tax + self.provincial_tax + additional_total
    }

    pub fn statutory_total(&self) -> Decimal {
        self.cpp + self.cpp2 + self.ei + self.federal_tax + self.provincial_tax
    }
}

impl Default for Deductions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalDeduction {
    pub name: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalEarning {
    pub id: Option<i64>,
    pub payroll_id: i64,
    pub earning_type: String,
    pub amount: Decimal,
    pub hours: Option<Decimal>,
    /// Whether this is a periodic payment (true) or non-periodic payment (false)
    /// Periodic: allowance, benefit, holiday, overtime (same period), vacation (if taken)
    /// Non-periodic: commission, bonus, retroactive pay, accumulated overtime
    pub is_periodic: bool,
}

impl AdditionalEarning {
    /// Determine if an earning type is periodic or non-periodic based on CRA T4127 definitions
    pub fn is_periodic_earning_type(earning_type: &str) -> bool {
        EarningType::from_str(earning_type).map(|et| et.is_periodic()).unwrap_or(true)
        // Default to periodic for unknown types
    }
}

/// Central enum for additional earning types
/// Used for CSV import/export, UI display, and autofill
/// Pre-defined types (also defined in src/types/payroll.ts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EarningType {
    Allowance,
    Benefit,
    Bonus,
    Commission,
    Other,
    Vacation,
}

impl EarningType {
    /// Get the canonical name string for storage
    pub fn as_str(&self) -> &'static str {
        match self {
            EarningType::Allowance => "allowance",
            EarningType::Benefit => "benefit",
            EarningType::Bonus => "bonus",
            EarningType::Commission => "commission",
            EarningType::Other => "other",
            EarningType::Vacation => "vacation",
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            EarningType::Allowance => "Allowance",
            EarningType::Benefit => "Benefit",
            EarningType::Bonus => "Bonus",
            EarningType::Commission => "Commission",
            EarningType::Other => "Other",
            EarningType::Vacation => "Vacation",
        }
    }

    /// Check if this is a periodic or non-periodic earning based on CRA T4127 definitions
    /// also update (src/types/payroll.ts)
    /// Periodic payments (I in T4127): allowance, benefit, vacation
    pub fn is_periodic(&self) -> bool {
        match self {
            EarningType::Allowance => true,
            EarningType::Benefit => true,
            EarningType::Bonus => false,
            EarningType::Commission => false,
            EarningType::Other => false, // Default to non-periodic for "other"
            EarningType::Vacation => true,
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase().replace(' ', "_");
        match lower.as_str() {
            "allowance" => Some(EarningType::Allowance),
            "benefit" => Some(EarningType::Benefit),
            "bonus" => Some(EarningType::Bonus),
            "commission" => Some(EarningType::Commission),
            "other" => Some(EarningType::Other),
            "vacation" => Some(EarningType::Vacation),
            _ => None,
        }
    }

    /// Get all known earning type names
    pub fn all_names() -> Vec<&'static str> {
        vec!["allowance", "benefit", "bonus", "commission", "other", "vacation"]
    }
}

/// Central enum for additional deduction types
/// also update (src/types/payroll.ts)
/// Each variant maps to a specific purpose:
/// - T4127 tax form variables
/// - CSV import/export headers
/// - UI display labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionType {
    /// Group/health insurance (not reported on T4127)
    GroupInsurance,
    /// RPP/RRSP contributions - maps to T4127 variable F
    PensionRrsp,
    /// Union dues - maps to T4127 variable U1
    UnionDues,
    /// Net pay adjustment (not reported on T4127)
    NetPayAdjust,
    /// Additional tax withholding - maps to T4127 variable L
    AddonTax,
}

impl DeductionType {
    /// Get the canonical name string for storage
    pub fn as_str(&self) -> &'static str {
        match self {
            DeductionType::GroupInsurance => "group_insurance",
            DeductionType::PensionRrsp => "pension_rrsp",
            DeductionType::UnionDues => "union_dues",
            DeductionType::NetPayAdjust => "net_pay_adjust",
            DeductionType::AddonTax => "addon_tax",
        }
    }

    /// Get T4127 variable code if applicable
    pub fn t4127_variable(&self) -> Option<&'static str> {
        match self {
            DeductionType::PensionRrsp => Some("F"),
            DeductionType::UnionDues => Some("U1"),
            DeductionType::AddonTax => Some("L"),
            _ => None,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            DeductionType::GroupInsurance => "Group Insurance",
            DeductionType::PensionRrsp => "Pension/RRSP",
            DeductionType::UnionDues => "Union Dues",
            DeductionType::NetPayAdjust => "Net Pay Adjustment",
            DeductionType::AddonTax => "Additional Tax",
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase().replace(' ', "_");
        match lower.as_str() {
            "group_insurance" => Some(DeductionType::GroupInsurance),
            "pension_rrsp" | "rpp" => Some(DeductionType::PensionRrsp),
            "union_dues" => Some(DeductionType::UnionDues),
            "net_pay_adjust" => Some(DeductionType::NetPayAdjust),
            "addon_tax" | "additional_tax" => Some(DeductionType::AddonTax),
            _ => None,
        }
    }

    /// Get all known deduction type names
    pub fn all_names() -> Vec<&'static str> {
        vec!["group_insurance", "pension_rrsp", "union_dues", "net_pay_adjust", "addon_tax"]
    }
}

/// T4 box type identifiers for flexible box value storage
/// Used for CSV import/export, UI display, and T4 slip generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum T4BoxType {
    Box14,  // Employment income
    Box16,  // CPP contributions
    Box16A, // CPP2 contributions
    Box18,  // EI premiums
    Box20,  // RPP contributions
    Box22,  // Income tax deducted
    Box24,  // EI insurable earnings
    Box26,  // CPP pensionable earnings
    Box45,  // Employer-offered dental benefit
    Box52,  // Pension adjustment
}

impl T4BoxType {
    /// Get the canonical name string for storage (matches T4127/CSV format)
    pub fn as_str(&self) -> &'static str {
        match self {
            T4BoxType::Box14 => "box_14",
            T4BoxType::Box16 => "box_16",
            T4BoxType::Box16A => "box_16a",
            T4BoxType::Box18 => "box_18",
            T4BoxType::Box20 => "box_20",
            T4BoxType::Box22 => "box_22",
            T4BoxType::Box24 => "box_24",
            T4BoxType::Box26 => "box_26",
            T4BoxType::Box45 => "box_45",
            T4BoxType::Box52 => "box_52",
        }
    }

    /// Get the CRA T4 box number
    pub fn box_number(&self) -> i32 {
        match self {
            T4BoxType::Box14 => 14,
            T4BoxType::Box16 => 16,
            T4BoxType::Box16A => 16, // Box 16A (CPP2)
            T4BoxType::Box18 => 18,
            T4BoxType::Box20 => 20,
            T4BoxType::Box22 => 22,
            T4BoxType::Box24 => 24,
            T4BoxType::Box26 => 26,
            T4BoxType::Box45 => 45,
            T4BoxType::Box52 => 52,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            T4BoxType::Box14 => "Employment income",
            T4BoxType::Box16 => "CPP contributions",
            T4BoxType::Box16A => "CPP2 contributions",
            T4BoxType::Box18 => "EI premiums",
            T4BoxType::Box20 => "RPP contributions",
            T4BoxType::Box22 => "Income tax deducted",
            T4BoxType::Box24 => "EI insurable earnings",
            T4BoxType::Box26 => "CPP pensionable earnings",
            T4BoxType::Box45 => "Dental benefit",
            T4BoxType::Box52 => "Pension adjustment",
        }
    }

    /// Parse from box code string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        let lower = s.to_lowercase().replace('-', "_");
        match lower.as_str() {
            "box_14" | "box14" => Some(T4BoxType::Box14),
            "box_16" | "box16" => Some(T4BoxType::Box16),
            "box_16a" | "box16a" => Some(T4BoxType::Box16A),
            "box_18" | "box18" => Some(T4BoxType::Box18),
            "box_20" | "box20" => Some(T4BoxType::Box20),
            "box_22" | "box22" => Some(T4BoxType::Box22),
            "box_24" | "box24" => Some(T4BoxType::Box24),
            "box_26" | "box26" => Some(T4BoxType::Box26),
            "box_45" | "box45" => Some(T4BoxType::Box45),
            "box_52" | "box52" => Some(T4BoxType::Box52),
            _ => None,
        }
    }

    /// Parse from box number
    pub fn from_box_number(num: i32) -> Option<Self> {
        match num {
            14 => Some(T4BoxType::Box14),
            16 => Some(T4BoxType::Box16),
            18 => Some(T4BoxType::Box18),
            20 => Some(T4BoxType::Box20),
            22 => Some(T4BoxType::Box22),
            24 => Some(T4BoxType::Box24),
            26 => Some(T4BoxType::Box26),
            45 => Some(T4BoxType::Box45),
            52 => Some(T4BoxType::Box52),
            _ => None,
        }
    }

    /// Get all known T4 box type names
    pub fn all_names() -> Vec<&'static str> {
        vec!["box_14", "box_16", "box_16a", "box_18", "box_20", "box_22", "box_24", "box_26", "box_45", "box_52"]
    }
}

impl Payroll {
    pub fn calculate_gross_pay(&mut self) {
        let additional_total: Decimal = self.additional_earnings.iter().map(|e| e.amount).sum();
        self.additional_earnings_total = additional_total;
        self.gross_pay += additional_total;
    }

    /// Calculate insurable earnings (IE in T4127)
    /// Per CRA rules: ALL earnings are insurable (periodic + non-periodic)
    /// This includes: regular pay, overtime, allowance, benefit, holiday, commission, bonus, etc.
    pub fn calculate_insured_earning(&mut self) {
        // All earnings are insurable per CRA T4127
        self.insured_earning = self.gross_pay;
    }

    /// Get periodic income (I in T4127) - used for tax calculations
    /// Includes: regular pay, overtime (same period), allowance, benefit, holiday, vacation (if taken)
    pub fn periodic_income(&self) -> Decimal {
        let periodic_additional: Decimal = self.additional_earnings.iter().filter(|e| e.is_periodic).map(|e| e.amount).sum();

        // Base gross_pay minus all additional, then add back periodic additional
        self.gross_pay - self.additional_earnings_total + periodic_additional
    }

    /// Get non-periodic income (B/I1 in T4127) - used for tax calculations
    /// Includes: commission, bonus, retroactive pay, accumulated overtime
    pub fn non_periodic_income(&self) -> Decimal {
        self.additional_earnings.iter().filter(|e| !e.is_periodic).map(|e| e.amount).sum()
    }

    pub fn calculate_net_pay(&mut self) {
        self.total_deductions = self.deductions.total();
        self.additional_deductions = self.deductions.additional.iter().map(|d| d.amount).sum();
        self.net_pay = self.gross_pay + self.additional_earnings_total - self.total_deductions;
    }

    pub fn validate(&self) -> crate::Result<()> {
        // Validate that there is some earnings (gross pay or additional earnings)
        if self.gross_pay <= Decimal::ZERO && self.additional_earnings_total <= Decimal::ZERO {
            return Err(crate::PayrollError::InvalidPayAmount("Gross pay or additional earnings must be greater than zero".to_string()));
        }

        // Validate net pay is not negative
        if self.net_pay < Decimal::ZERO {
            return Err(crate::PayrollError::InvalidPayAmount("Net pay cannot be negative".to_string()));
        }

        // Validate hours are non-negative
        if let Some(hours) = self.regular_hours {
            if hours < Decimal::ZERO {
                return Err(crate::PayrollError::InvalidPayAmount("Regular hours cannot be negative".to_string()));
            }
        }
        if let Some(hours) = self.overtime_hours {
            if hours < Decimal::ZERO {
                return Err(crate::PayrollError::InvalidPayAmount("Overtime hours cannot be negative".to_string()));
            }
        }

        // Validate additional earnings
        for earning in &self.additional_earnings {
            if earning.amount < Decimal::ZERO {
                return Err(crate::PayrollError::InvalidPayAmount(format!("Earning amount cannot be negative: {}", earning.earning_type)));
            }
            if let Some(hours) = earning.hours {
                if hours < Decimal::ZERO {
                    return Err(crate::PayrollError::InvalidPayAmount(format!("Earning hours cannot be negative: {}", earning.earning_type)));
                }
            }
        }

        // Validate date range
        if self.pay_period_end < self.pay_period_start {
            return Err(crate::PayrollError::InvalidDateRange("Pay period end cannot be before start".to_string()));
        }

        if self.pay_date < self.pay_period_end {
            return Err(crate::PayrollError::InvalidDateRange("Pay date should not be before pay period end".to_string()));
        }

        Ok(())
    }
}

/// Year-to-date reduction calculation values (before current pay period)
/// These values are used in tax reduction formulas per t4127.md
/// Stored as type/value pairs for flexibility (e.g., B1, D, DQ, D1, D2, D2Q, M, M1, F4, PIYTD, IEYTD, GYTD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdReductionValue {
    pub id: Option<i64>,
    pub payroll_id: i64,
    pub ytd_type: String, // B1, D, DQ, D1, D2, D2Q, M, M1, F4, PIYTD, IEYTD, GYTD
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}

impl YtdReductionValue {
    pub fn new(payroll_id: i64, ytd_type: String, amount: Decimal) -> Self {
        Self { id: None, payroll_id, ytd_type, amount, created_at: Utc::now() }
    }
}

/// Common YTD type constants from t4127.md
pub mod ytd_types {
    pub const B1_NONPERIODIC: &str = "B1"; // Non-periodic payments (bonuses, retroactive pay, etc.)
    pub const D_CPP: &str = "D"; // Base CPP contributions
    pub const DQ_QPP: &str = "DQ"; // Base QPP contributions
    pub const D1_EI: &str = "D1"; // EI premiums
    pub const D2_CPP2: &str = "D2"; // Second additional CPP contributions
    pub const D2Q_QPP2: &str = "D2Q"; // Second additional QPP contributions
    pub const M_TAX: &str = "M"; // Accumulated federal+provincial tax deductions
    pub const M1_TAX_ON_B1: &str = "M1"; // Tax deducted on non-periodic payments
    pub const F4_PENSION_RRSP: &str = "F4"; // RPP/RRSP contributions from non-periodic payments
    pub const PIYTD_PENSIONABLE: &str = "PIYTD"; // Pensionable earnings YTD
    pub const IEYTD_INSURABLE: &str = "IEYTD"; // Insurable earnings YTD
    pub const GYTD_GROSS: &str = "GYTD"; // Gross earnings YTD
}

/// Year-to-date totals for an employee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtdTotals {
    pub employee_id: i64,
    pub year: i32,
    pub gross_pay: Decimal,
    pub cpp: Decimal,
    pub cpp2: Decimal,
    pub ei: Decimal,
    pub federal_tax: Decimal,
    pub provincial_tax: Decimal,
    pub net_pay: Decimal,
    /// Box 20 - RPP contributions (sum of PensionRrsp deductions from history)
    pub rpp_contributions: Decimal,
    /// Box 52 - Pension adjustment (sum of UnionDues deductions from history)
    pub pension_adjustment: Decimal,
}

impl YtdTotals {
    pub fn new(employee_id: i64, year: i32) -> Self {
        Self {
            employee_id,
            year,
            gross_pay: Decimal::ZERO,
            cpp: Decimal::ZERO,
            cpp2: Decimal::ZERO,
            ei: Decimal::ZERO,
            federal_tax: Decimal::ZERO,
            provincial_tax: Decimal::ZERO,
            net_pay: Decimal::ZERO,
            rpp_contributions: Decimal::ZERO,
            pension_adjustment: Decimal::ZERO,
        }
    }
}
