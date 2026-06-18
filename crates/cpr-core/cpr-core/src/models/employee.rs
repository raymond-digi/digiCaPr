use chrono::{DateTime, Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: Option<i64>,
    pub is_active: bool,
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub sin: String, // Social Insurance Number
    pub address: Address,
    pub pay_type: PayType,
    pub pay_rate: Decimal,
    pub notes: Option<String>,
    
    pub date_of_birth: NaiveDate,
    pub vacation_pay_rate: Decimal,    // e.g., 0.04 for 4%, 0.06 for 6%
    pub vacation_balance: Decimal,     // Current vacation balance in dollars (fast lookup)
    pub vacation_balance_days: Decimal, // Current vacation balance in days (for non-hourly employees)
    pub overtime_multiplier: Decimal,   // e.g., 1.5 for time-and-a-half
    
    pub ei_exempt: bool,  // Whether employee is exempt from EI deductions
    pub cpp_exempt: bool,  // Whether employee is exempt from CPP deductions
    
    pub hire_date: NaiveDate,
    pub hire_province: String,  // Province for tax calculations (independent of address_province)
    pub termination_date: Option<NaiveDate>,
    /// Employer-offered dental benefit code for T4 Box 45
    /// 1 = No dental benefit
    /// 2 = Basic dental only
    /// 3 = Comprehensive dental
    pub dental_benefit: i32,  // 1, 2, or 3 (default: 1)
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Province {
    ON, // Ontario
    QC, // Quebec
    BC, // British Columbia
    AB, // Alberta
    SK, // Saskatchewan
    MB, // Manitoba
    NS, // Nova Scotia
    NB, // New Brunswick
    PE, // Prince Edward Island
    NL, // Newfoundland and Labrador
    NT, // Northwest Territories
    YT, // Yukon
    NU, // Nunavut
}

impl Province {
    pub fn all() -> Vec<Province> {
        vec![
            Province::ON,
            Province::QC,
            Province::BC,
            Province::AB,
            Province::SK,
            Province::MB,
            Province::NS,
            Province::NB,
            Province::PE,
            Province::NL,
            Province::NT,
            Province::YT,
            Province::NU,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Province::ON => "Ontario",
            Province::QC => "Quebec",
            Province::BC => "British Columbia",
            Province::AB => "Alberta",
            Province::SK => "Saskatchewan",
            Province::MB => "Manitoba",
            Province::NS => "Nova Scotia",
            Province::NB => "New Brunswick",
            Province::PE => "Prince Edward Island",
            Province::NL => "Newfoundland and Labrador",
            Province::NT => "Northwest Territories",
            Province::YT => "Yukon",
            Province::NU => "Nunavut",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Province::ON => "ON",
            Province::QC => "QC",
            Province::BC => "BC",
            Province::AB => "AB",
            Province::SK => "SK",
            Province::MB => "MB",
            Province::NS => "NS",
            Province::NB => "NB",
            Province::PE => "PE",
            Province::NL => "NL",
            Province::NT => "NT",
            Province::YT => "YT",
            Province::NU => "NU",
        }
    }

    pub fn from_code(code: &str) -> Option<Province> {
        match code.to_uppercase().as_str() {
            "ON" => Some(Province::ON),
            "QC" => Some(Province::QC),
            "BC" => Some(Province::BC),
            "AB" => Some(Province::AB),
            "SK" => Some(Province::SK),
            "MB" => Some(Province::MB),
            "NS" => Some(Province::NS),
            "NB" => Some(Province::NB),
            "PE" => Some(Province::PE),
            "NL" => Some(Province::NL),
            "NT" => Some(Province::NT),
            "YT" => Some(Province::YT),
            "NU" => Some(Province::NU),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayType {
    Hourly,
    Weekly,
    Monthly,
    Annual,
}

impl PayType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayType::Hourly => "Hourly",
            PayType::Weekly => "Weekly",
            PayType::Monthly => "Monthly",
            PayType::Annual => "Annual",
        }
    }

    pub fn from_str(s: &str) -> Option<PayType> {
        match s.to_lowercase().as_str() {
            "hourly" => Some(PayType::Hourly),
            "weekly" => Some(PayType::Weekly),
            "monthly" => Some(PayType::Monthly),
            "annual" => Some(PayType::Annual),
            _ => None,
        }
    }
}


impl Employee {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Calculate employee's age on a given date
    pub fn age_on_date(&self, date: NaiveDate) -> i32 {
        let mut years = date.year() - self.date_of_birth.year();
        
        // Adjust if birthday hasn't occurred yet this year
        if date.month() <  self.date_of_birth.month()
            || (date.month() == self.date_of_birth.month() && date.day() < self.date_of_birth.day()) {
            years -= 1;
        }
        
        years
    }
    
    /// Check if employee is CPP-eligible on a given date (age 18-70)
    pub fn is_cpp_eligible(&self, date: NaiveDate) -> bool {
        let age = self.age_on_date(date);
        age >= 18 && age <= 70
    }

    pub fn validate(&self) -> crate::Result<()> {
        // Validate SIN format (9 digits)
        let sin_digits: String = self.sin.chars().filter(|c| c.is_ascii_digit()).collect();
        if sin_digits.len() != 9 {
            return Err(crate::PayrollError::InvalidSin(
                "SIN must be 9 digits".to_string()
            ));
        }

        // Validate pay rate is positive
        if self.pay_rate <= Decimal::ZERO {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Pay rate must be positive".to_string()
            ));
        }

        // Validate dates
        if let Some(term_date) = self.termination_date {
            if term_date < self.hire_date {
                return Err(crate::PayrollError::InvalidDateRange(
                    "Termination date cannot be before hire date".to_string()
                ));
            }
        }
        
        // Validate date of birth is in the past
        let today = chrono::Local::now().naive_local().date();
        if self.date_of_birth >= today {
            return Err(crate::PayrollError::InvalidDateRange(
                "Date of birth must be in the past".to_string()
            ));
        }
        
        // Validate reasonable age (15-100)
        let age = self.age_on_date(today);
        if age < 15 || age > 100 {
            return Err(crate::PayrollError::InvalidDateRange(
                format!("Employee age ({}) must be between 15 and 100", age)
            ));
        }
        
        // Validate vacation pay rate (0-20%)
        if self.vacation_pay_rate < Decimal::ZERO || self.vacation_pay_rate > rust_decimal_macros::dec!(0.20) {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Vacation pay rate must be between 0% and 20%".to_string()
            ));
        }
        
        // Validate overtime multiplier (1.0-3.0)
        if self.overtime_multiplier < rust_decimal_macros::dec!(1.0) || self.overtime_multiplier > rust_decimal_macros::dec!(3.0) {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Overtime multiplier must be between 1.0 and 3.0".to_string()
            ));
        }

        // Validate address province
        let addr_prov_code = self.address.province.trim();
        if addr_prov_code.is_empty() {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Address province is required".to_string()
            ));
        }

        
        Ok(())
    }
}

/// Autofill type for employee autofill values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutofillType {
    Earning,
    Deduction,
}

impl AutofillType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AutofillType::Earning => "earning",
            AutofillType::Deduction => "deduction",
        }
    }

    pub fn from_str(s: &str) -> Option<AutofillType> {
        match s.to_lowercase().as_str() {
            "earning" => Some(AutofillType::Earning),
            "deduction" => Some(AutofillType::Deduction),
            _ => None,
        }
    }
}

/// Employee autofill values for additional earnings and deductions
/// These are default values that auto-populate when creating payroll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeAutofill {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub autofill_type: AutofillType,
    pub type_name: String,
    pub amount: Decimal,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

impl EmployeeAutofill {
    pub fn new(employee_id: i64, autofill_type: AutofillType, type_name: String, amount: Decimal) -> Self {
        Self {
            id: None,
            employee_id,
            autofill_type,
            type_name,
            amount,
            is_active: true,
            created_at: Some(Utc::now()),
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.type_name.trim().is_empty() {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Type name cannot be empty".to_string()
            ));
        }

        if self.amount < Decimal::ZERO {
            return Err(crate::PayrollError::InvalidPayAmount(
                "Amount cannot be negative".to_string()
            ));
        }

        Ok(())
    }
}
