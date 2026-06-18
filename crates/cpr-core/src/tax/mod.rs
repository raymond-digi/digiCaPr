use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::models::Province;
use crate::models::payroll::YtdTotals;

pub mod config;
pub mod t4127;

// Primary API - T4127Context with factory method
pub use t4127::T4127Context;

// Config loading function
pub use config::load_tax_config;

// Config update functions (async)
pub use config::check_github_update;
pub use config::download_github_config;
pub use config::update_if_available;

// App updater module (requires "app-updater" feature)
#[cfg(feature = "app-updater")]
pub use config::app_updater;

/// Trait for retrieving year-to-date totals for tax calculations
/// This allows to retrieve YTD values without depending on the database layer
pub trait YtdProvider {
    /// Get YTD totals for an employee for a specific year
    fn get_ytd_totals(&self, employee_id: i64, year: i32) -> Result<YtdTotals, Box<dyn std::error::Error + Send + Sync>>;
}

/// Employee information needed for tax calculations
/// This struct contains the employee-specific data that doesn't change during a payroll run
#[derive(Debug, Clone)]
pub struct EmployeeInfo {
    pub date_of_birth: NaiveDate,
    pub province: Province,
    pub cpp_exempt: bool,
    pub ei_exempt: bool,
    pub federal_personal_amount: Decimal,
    pub provincial_personal_amount: Decimal,
}

impl EmployeeInfo {
    /// Calculate the employee's age as of a given date
    pub fn age_as_of(&self, date: NaiveDate) -> i32 {
        let mut age = date.year() - self.date_of_birth.year();
        // Adjust if birthday hasn't occurred yet this year
        if date.month() < self.date_of_birth.month()
            || (date.month() == self.date_of_birth.month() && date.day() < self.date_of_birth.day())
        {
            age -= 1;
        }
        age
    }
}

/// Trait for retrieving employee information for tax calculations
/// This allows to retrieve employee data without depending on the database layer
pub trait EmployeeProvider {
    /// Get employee information needed for tax calculations
    /// The year parameter is needed to retrieve the correct personal amounts
    fn get_employee_info(&self, employee_id: i64, year: i32) -> Result<EmployeeInfo, Box<dyn std::error::Error + Send + Sync>>;
}

/// Tax year configuration
#[derive(Debug, Clone)]
pub struct TaxYearConfig {
    pub year: i32,
    pub cpp: CppConfig,
    pub cpp2: Cpp2Config,
    pub ei: EiConfig,
    pub federal: FederalTaxConfig,
    pub provincial: ProvincialTaxConfig,
}

#[derive(Debug, Clone)]
pub struct CppConfig {
    pub basic_exemption: Decimal,
    pub max_pensionable_earnings: Decimal,
    pub base_rate: Decimal,
    pub first_additional_rate: Decimal,
    pub employee_rate: Decimal,
    pub max_contribution: Decimal,
    pub max_base_contribution: Decimal,
}

#[derive(Debug, Clone)]
pub struct Cpp2Config {
    pub rate: Decimal,
    pub max_earnings: Decimal,
    pub max_contribution: Decimal,
}

#[derive(Debug, Clone)]
pub struct EiConfig {
    pub max_insurable_earnings: Decimal,
    pub employee_rate: Decimal,
    pub qc_employee_rate: Decimal, // Quebec has different rate due to QPIP
    pub max_contribution: Decimal,
    pub qc_max_contribution: Decimal,
}

#[derive(Debug, Clone)]
pub struct FederalTaxConfig {
    pub basic_personal_amount: Decimal,
    pub canada_employment_amount: Decimal,
    pub brackets: Vec<TaxBracket>,
}

#[derive(Debug, Clone)]
pub struct ProvincialTaxConfig {
    pub province_configs: HashMap<Province, ProvinceTaxRates>,
}

#[derive(Debug, Clone)]
pub struct ProvinceTaxRates {
    pub basic_personal_amount: Decimal,
    pub canada_employment_amount: Decimal,
    pub brackets: Vec<TaxBracket>,
    pub surtax: Option<Surtax>,
    pub surtax_tiers: Vec<SurtaxTier>,
    /// Alberta-specific K5P tax reduction threshold (only for AB)
    pub k5p_threshold: Option<Decimal>,
    /// Alberta-specific K5P tax reduction rate (only for AB)
    pub k5p_rate: Option<Decimal>,
    /// S2 amount for provincial tax reduction (2026 - ON=300, BC=575)
    pub s2_amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct TaxBracket {
    pub lower_limit: Decimal,
    pub upper_limit: Option<Decimal>, // None for top bracket
    pub rate: Decimal,
    pub constant: Decimal, // K constant for simplified tax calculation (R × A - K)
}

#[derive(Debug, Clone)]
pub struct Surtax {
    pub threshold: Decimal,
    pub rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct SurtaxTier {
    pub threshold: Decimal,
    pub rate: Decimal,
}

/// Combined tax calculation result
#[derive(Debug, Clone)]
pub struct PayrollDeductions {
    pub cpp: Decimal,
    pub cpp2: Decimal,
    pub ei: Decimal,
    pub federal_tax: Decimal,
    pub provincial_tax: Decimal,
}

impl PayrollDeductions {
    pub fn total(&self) -> Decimal {
        self.cpp + self.cpp2 + self.ei + self.federal_tax + self.provincial_tax
    }
}
