use crate::error::CommandError;
use crate::state::AppState;
use chrono::{Datelike, NaiveDate, Utc};
use cpr_core::models::payroll::YtdTotals;
use cpr_core::models::vacation::{calculate_vacation_accrual, calculate_vacation_accrual_days, VacationAccrual, VacationTransactionType};
use cpr_core::models::{DeductionType, Deductions, EarningType, PayType, Payroll};
use cpr_core::tax::{EmployeeInfo, EmployeeProvider, T4127Context, YtdProvider};
use cpr_db::repository::history::PayrollHistoryFilter;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use tauri::{AppHandle, State};

/// YtdProvider implementation that wraps the database connection
struct DbYtdProvider<'a> {
    db: &'a cpr_db::repository::Database,
}

impl<'a> DbYtdProvider<'a> {
    fn new(db: &'a cpr_db::repository::Database) -> Self {
        Self { db }
    }
}

impl<'a> YtdProvider for DbYtdProvider<'a> {
    fn get_ytd_totals(&self, employee_id: i64, year: i32) -> Result<YtdTotals, Box<dyn std::error::Error + Send + Sync>> {
        self.db.payroll_history().get_ytd_totals(employee_id, year).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

/// EmployeeProvider implementation that wraps the database connection
struct DbEmployeeProvider<'a> {
    db: &'a cpr_db::repository::Database,
}

impl<'a> DbEmployeeProvider<'a> {
    fn new(db: &'a cpr_db::repository::Database) -> Self {
        Self { db }
    }
}

impl<'a> EmployeeProvider for DbEmployeeProvider<'a> {
    fn get_employee_info(&self, employee_id: i64, year: i32) -> Result<EmployeeInfo, Box<dyn std::error::Error + Send + Sync>> {
        let employee = self.db.employees().get(employee_id).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let province_code = employee.hire_province.trim().to_string();
        let province = cpr_core::models::Province::from_code(&province_code)
            .ok_or_else(|| format!("Invalid province code: {}", province_code))
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)) as Box<dyn std::error::Error + Send + Sync>)?;

        // Get personal amount with fallback to previous years (with indexing if needed)
        let (federal_pa, provincial_pa) = self
            .db
            .personal_amount()
            .get_personal_amount_with_fallback(employee_id, &province_code, year)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(EmployeeInfo {
            date_of_birth: employee.date_of_birth,
            province,
            cpp_exempt: employee.cpp_exempt,
            ei_exempt: employee.ei_exempt,
            federal_personal_amount: federal_pa,
            provincial_personal_amount: provincial_pa,
        })
    }
}

fn get_pay_periods_per_year(period_days: i64) -> i32 {
    let days = period_days as f64;
    if days < 8.0 {
        52
    } else if days < 15.0 {
        26
    } else if days < 16.0 {
        24
    } else {
        12
    }
}

/// Helper to get database from state

fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&cpr_db::repository::Database) -> Result<T, CommandError>,
{
    let db_guard = state.database.lock().unwrap();
    let db = db_guard.as_ref().ok_or_else(|| CommandError::new("No database open"))?;
    f(db)
}

/// Calculate payroll for an employee
/// This performs the tax calculations but doesn't save to database
#[tauri::command]
pub async fn calculate_payroll(
    employee_id: i64,
    pay_period_start: String,
    pay_period_end: String,
    pay_date: String,
    regular_hours: Option<f64>,
    overtime_hours: Option<f64>,
    gross_pay: Option<f64>,
    additional_earnings: Option<Vec<cpr_core::models::payroll::AdditionalEarning>>,
    additional_deductions: Option<Vec<cpr_core::models::payroll::AdditionalDeduction>>,
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Payroll, CommandError> {
    with_db(&state, |db| {
        // Get employee
        let employee = db.employees().get(employee_id)?;

        // Parse dates
        let period_start = NaiveDate::parse_from_str(&pay_period_start, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let period_end = NaiveDate::parse_from_str(&pay_period_end, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let payment_date = NaiveDate::parse_from_str(&pay_date, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        let period_duration = period_end - period_start;
        let period_days_i64 = period_duration.num_days() + 1;
        let period_days_dec = Decimal::from(period_days_i64);
        let year_days_dec = Decimal::new(36525, 2);
        let period_fraction = period_days_dec / year_days_dec;
        let annual_hours_dec = Decimal::new(2080, 0);
        let pay_periods_per_year = get_pay_periods_per_year(period_days_i64);
        let pay_year = payment_date.year() as i32;

        // Parse regular and overtime hours to Decimal
        let regular_decimal: Option<Decimal> =
            regular_hours.map(|h| Decimal::from_str(&h.to_string()).map_err(|e| CommandError::new(format!("Invalid regular hours: {}", e)))).transpose()?;

        let overtime_decimal: Option<Decimal> =
            overtime_hours.map(|h| Decimal::from_str(&h.to_string()).map_err(|e| CommandError::new(format!("Invalid overtime hours: {}", e)))).transpose()?;

        // Calculate gross pay if not provided
        let calculated_gross = if let Some(gross_f64) = gross_pay {
            Decimal::from_str(&gross_f64.to_string()).map_err(|e| CommandError::new(format!("Invalid gross pay: {}", e)))?
        } else {
            match employee.pay_type {
                cpr_core::models::PayType::Annual => employee.pay_rate / Decimal::from(pay_periods_per_year as i64),
                cpr_core::models::PayType::Weekly => employee.pay_rate * Decimal::from(52) / Decimal::from(pay_periods_per_year as i64),
                cpr_core::models::PayType::Monthly => (employee.pay_rate * Decimal::from(12)) / Decimal::from(pay_periods_per_year as i64),
                cpr_core::models::PayType::Hourly => {
                    let reg_hours = regular_decimal.unwrap_or(annual_hours_dec * period_fraction);
                    let ot_hours = overtime_decimal.unwrap_or(Decimal::ZERO);
                    reg_hours * employee.pay_rate + ot_hours * employee.pay_rate * employee.overtime_multiplier
                }
            }
        };

        // Process additional earnings if provided
        let earnings_vec = additional_earnings.unwrap_or_default();

        // Calculate totals for additional earnings
        let additional_earnings_total: Decimal = earnings_vec.iter().map(|e| e.amount).sum();

        // Separate periodic and non-periodic earnings
        let periodic_additional: Decimal = earnings_vec.iter().filter(|e| e.is_periodic).map(|e| e.amount).sum();

        let non_periodic_additional: Decimal = earnings_vec.iter().filter(|e| !e.is_periodic).map(|e| e.amount).sum();

        // Total earnings = base gross + all additional
        let total_gross = calculated_gross + additional_earnings_total;

        // Periodic income = base gross + periodic additional (I in T4127)
        let periodic_income = calculated_gross + periodic_additional;

        // Create T4127 context for the year
        let mut tax_ctx = T4127Context::new(payment_date.year())
            .map_err(|e| CommandError::new(format!("Failed to load tax config for year {} : {}", payment_date.year(), e.to_string())))?;

        // Calculate all deductions using unified T4127 context
        // This ensures CPP, EI, and taxes are calculated together with proper interdependencies
        // The method retrieves YTD values and employee info internally using the providers
        let ytd_provider = DbYtdProvider::new(db);
        let employee_provider = DbEmployeeProvider::new(db);
        let employee_info = employee_provider.get_employee_info(employee_id, pay_year)?;
        let additional_deductions_vec = additional_deductions.unwrap_or_default();
        let payroll_deductions = tax_ctx.calculate_all_deductions(
            employee_id,
            periodic_income,
            non_periodic_additional,
            pay_periods_per_year,
            payment_date,
            &ytd_provider,
            &employee_provider,
            &additional_deductions_vec,
        )?;

        let deductions = Deductions {
            cpp: payroll_deductions.cpp,
            cpp2: payroll_deductions.cpp2,
            ei: payroll_deductions.ei,
            federal_tax: payroll_deductions.federal_tax,
            provincial_tax: payroll_deductions.provincial_tax,
            additional: additional_deductions_vec,
        };

        let total_deductions_val = deductions.total();
        let net_pay = total_gross - total_deductions_val;

        // Calculate pay period number
        let year_start = NaiveDate::from_ymd_opt(pay_year, 1, 1).ok_or_else(|| CommandError::new("Invalid year start date"))?;
        let days_since_year_start = period_start.signed_duration_since(year_start).num_days();
        let pay_period_number = std::cmp::max((days_since_year_start / period_days_i64) as i32 + 1, 1i32);

        // Create payroll record
        let payroll = Payroll {
            id: None,
            employee_id,
            pay_period_start: period_start,
            pay_period_end: period_end,
            pay_date: payment_date,
            regular_hours: regular_decimal,
            overtime_hours: overtime_decimal,
            additional_earnings: earnings_vec,
            insured_earning: total_gross,
            gross_pay: calculated_gross,
            additional_earnings_total,
            additional_tax_amount: Decimal::ZERO,
            deductions,
            net_pay,
            pay_period_number: Some(pay_period_number),
            total_pay_periods: pay_periods_per_year,
            total_deductions: total_deductions_val,
            additional_deductions: Decimal::ZERO,
            federal_personal_amount: employee_info.federal_personal_amount,
            province: employee_info.province.code().to_string(),
            provincial_personal_amount: employee_info.provincial_personal_amount,
            remittance_id: None,
            created_at: Utc::now(),
        };

        Ok(payroll)
    })
}

/// Save payroll to database
#[tauri::command]
pub async fn save_payroll(mut payroll: Payroll, state: State<'_, AppState>) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let id = db.payroll_history().create(&mut payroll)?;
        Ok(id)
    })
}

/// List all payroll records (limited implementation - by date range would be better)
#[tauri::command]
pub async fn list_payroll(state: State<'_, AppState>) -> Result<Vec<Payroll>, CommandError> {
    with_db(&state, |db| {
        // Get payroll for current year
        let current_year = chrono::Utc::now().year();
        let start_date = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap();
        let records = db.payroll_history().list_by_date_range(start_date, end_date)?;
        Ok(records)
    })
}

/// Filter options for listing payroll history
#[derive(Debug, serde::Deserialize)]
pub struct PayrollHistoryFilters {
    pub employee_id: Option<i64>,
    pub pay_date_from: Option<String>,
    pub pay_date_to: Option<String>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Result type for filtered payroll history list
#[derive(Debug, serde::Serialize)]
pub struct PayrollHistoryListResult {
    pub payrolls: Vec<Payroll>,
    pub total_count: i64,
}

/// List payroll history with filters for search functionality
#[tauri::command]
pub async fn list_payroll_history(filters: Option<PayrollHistoryFilters>, state: State<'_, AppState>) -> Result<PayrollHistoryListResult, CommandError> {
    with_db(&state, |db| {
        let filter = match filters {
            Some(f) => PayrollHistoryFilter {
                employee_id: f.employee_id,
                pay_date_from: f.pay_date_from.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                pay_date_to: f.pay_date_to.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                search_term: f.search_term,
                limit: f.limit,
                offset: f.offset,
            },
            None => PayrollHistoryFilter::default(),
        };

        let payrolls = db.payroll_history().list_filtered(&filter)?;
        let total_count = db.payroll_history().count_filtered(&filter)?;

        Ok(PayrollHistoryListResult { payrolls, total_count })
    })
}

/// List payroll records for a specific employee
#[tauri::command]
pub async fn list_employee_payroll(employee_id: i64, state: State<'_, AppState>) -> Result<Vec<Payroll>, CommandError> {
    with_db(&state, |db| {
        let records = db.payroll_history().list_by_employee(employee_id)?;
        Ok(records)
    })
}

/// Get a single payroll record by ID
#[tauri::command]
pub async fn get_payroll(id: i64, state: State<'_, AppState>) -> Result<Payroll, CommandError> {
    with_db(&state, |db| {
        let payroll = db.payroll_history().get(id)?;
        Ok(payroll)
    })
}

/// Get year-to-date totals for an employee
#[tauri::command]
pub async fn get_ytd_totals(employee_id: i64, year: i32, state: State<'_, AppState>) -> Result<cpr_core::models::payroll::YtdTotals, CommandError> {
    with_db(&state, |db| {
        let totals = db.payroll_history().get_ytd_totals(employee_id, year)?;
        Ok(totals)
    })
}

/// Create payroll for multiple employees
#[tauri::command]
pub async fn create_current_payroll(
    pay_period_start: String,
    pay_period_end: String,
    pay_date: String,
    employee_ids: Option<Vec<i64>>,
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CurrentPayrollResult, CommandError> {
    with_db(&state, |db| {
        let period_start = NaiveDate::parse_from_str(&pay_period_start, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let period_end = NaiveDate::parse_from_str(&pay_period_end, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let payment_date = NaiveDate::parse_from_str(&pay_date, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        let period_duration = period_end - period_start;
        let period_days_i64 = period_duration.num_days() + 1;
        let period_days_dec = Decimal::from(period_days_i64);
        let year_days_dec = Decimal::new(36525, 2);
        let period_fraction = period_days_dec / year_days_dec;
        let annual_hours_dec = Decimal::new(2080, 0);
        let pay_periods_per_year_i32 = get_pay_periods_per_year(period_days_i64);

        // Get employees to process
        let employees = if let Some(ids) = employee_ids {
            // Get specific employees
            let mut emps = Vec::new();
            for id in ids {
                match db.employees().get(id) {
                    Ok(emp) => emps.push(emp),
                    Err(_) => continue,
                }
            }
            emps
        } else {
            // Auto-add active salary and monthly employees only
            db.employees()
                .list_active()?
                .into_iter()
                .filter(|e| e.pay_type == PayType::Annual || e.pay_type == PayType::Weekly || e.pay_type == PayType::Monthly)
                .collect::<Vec<_>>()
        };

        let mut payrolls = Vec::new();
        let mut errors = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;

        for employee in employees {
            // Check if payroll already exists for this employee in current table (only one per employee)
            let existing_payrolls = db.payroll().list_by_employee(employee.id.unwrap())?;
            let existing_payroll = if existing_payrolls.is_empty() {
                None
            } else {
                // Take the first/only existing payroll for this employee
                Some(existing_payrolls[0].clone())
            };

            // Calculate payroll
            let pay_year = payment_date.year() as i32;

            if employee.pay_rate <= Decimal::ZERO {
                errors.push(CurrentPayrollError {
                    employee_id: employee.id.unwrap(),
                    employee_name: format!("{} {}", employee.first_name, employee.last_name),
                    error: "Pay rate must be greater than zero".to_string(),
                });
                continue;
            }

            // Calculate gross pay and hours based on pay type
            let (regular_decimal, overtime_decimal, calculated_gross) = match employee.pay_type {
                PayType::Annual => {
                    let calculated_gross = employee.pay_rate / Decimal::from(pay_periods_per_year_i32 as i64);
                    (None, None, calculated_gross)
                }
                PayType::Weekly => {
                    let calculated_gross = employee.pay_rate * Decimal::from(52) / Decimal::from(pay_periods_per_year_i32 as i64);
                    (None, None, calculated_gross)
                }
                PayType::Monthly => {
                    let calculated_gross = (employee.pay_rate * Decimal::from(12)) / Decimal::from(pay_periods_per_year_i32 as i64);
                    (None, None, calculated_gross)
                }
                PayType::Hourly => {
                    let reg_hours = annual_hours_dec * period_fraction;
                    let ot_hours = Decimal::ZERO;
                    let gross = reg_hours * employee.pay_rate + ot_hours * employee.pay_rate * employee.overtime_multiplier;
                    (Some(reg_hours), Some(ot_hours), gross)
                }
            };

            // Create T4127 context for the year
            let mut tax_ctx = T4127Context::new(payment_date.year())
                .map_err(|e| CommandError::new(format!("Failed to load tax config for year {} : {}", payment_date.year(), e.to_string())))?;

            let pay_periods_per_year = pay_periods_per_year_i32;

            // Calculate all deductions using unified T4127 context
            // This ensures CPP, EI, and taxes are calculated together with proper interdependencies
            // The method retrieves YTD values and employee info internally using the providers
            let ytd_provider = DbYtdProvider::new(db);
            let employee_provider = DbEmployeeProvider::new(db);
            let employee_info = employee_provider.get_employee_info(employee.id.unwrap(), pay_year)?;
            let additional_deductions_vec: Vec<cpr_core::models::payroll::AdditionalDeduction> = Vec::new();
            let payroll_deductions = tax_ctx.calculate_all_deductions(
                employee.id.unwrap(),
                calculated_gross,
                Decimal::ZERO, // No non-periodic earnings in create_current_payroll
                pay_periods_per_year,
                payment_date,
                &ytd_provider,
                &employee_provider,
                &additional_deductions_vec,
            )?;

            let deductions = Deductions {
                cpp: payroll_deductions.cpp,
                cpp2: payroll_deductions.cpp2,
                ei: payroll_deductions.ei,
                federal_tax: payroll_deductions.federal_tax,
                provincial_tax: payroll_deductions.provincial_tax,
                additional: additional_deductions_vec,
            };

            let total_deductions_val = deductions.total();
            let net_pay = calculated_gross - total_deductions_val;

            // Calculate pay period number
            let year_start = NaiveDate::from_ymd_opt(pay_year, 1, 1).ok_or_else(|| CommandError::new("Invalid year start date"))?;
            let days_since_year_start = period_start.signed_duration_since(year_start).num_days();
            let pay_period_number = std::cmp::max((days_since_year_start / period_days_i64) as i32 + 1, 1i32);

            // Build payroll with fixed dates from parameters
            let mut payroll = Payroll {
                id: None,
                employee_id: employee.id.unwrap(),
                pay_period_start: period_start,
                pay_period_end: period_end,
                pay_date: payment_date,
                regular_hours: regular_decimal,
                overtime_hours: overtime_decimal,
                additional_earnings: vec![],
                insured_earning: calculated_gross,
                gross_pay: calculated_gross,
                additional_earnings_total: Decimal::ZERO,
                additional_tax_amount: Decimal::ZERO,
                deductions,
                net_pay,
                pay_period_number: Some(pay_period_number),
                total_pay_periods: pay_periods_per_year,
                total_deductions: total_deductions_val,
                additional_deductions: Decimal::ZERO,
                federal_personal_amount: employee_info.federal_personal_amount,
                province: employee_info.province.code().to_string(),
                provincial_personal_amount: employee_info.provincial_personal_amount,
                remittance_id: None,
                created_at: Utc::now(),
            };

            // Determine if we're creating new or updating existing
            if let Some(existing) = existing_payroll {
                // Update existing payroll: preserve ID and created_at, overwrite other fields
                payroll.id = existing.id;
                payroll.created_at = existing.created_at;
                match db.payroll().update(&mut payroll) {
                    Ok(_) => {
                        payrolls.push(payroll);
                        updated_count += 1;
                    }
                    Err(e) => {
                        errors.push(CurrentPayrollError {
                            employee_id: employee.id.unwrap(),
                            employee_name: format!("{} {}", employee.first_name, employee.last_name),
                            error: format!("Failed to update current payroll: {}", e),
                        });
                    }
                }
            } else {
                // Create new payroll
                match db.payroll().create(&mut payroll) {
                    Ok(_) => {
                        payrolls.push(payroll);
                        created_count += 1;
                    }
                    Err(e) => {
                        errors.push(CurrentPayrollError {
                            employee_id: employee.id.unwrap(),
                            employee_name: format!("{} {}", employee.first_name, employee.last_name),
                            error: format!("Failed to save current payroll: {}", e),
                        });
                    }
                }
            }
        }

        Ok(CurrentPayrollResult { payrolls, errors, created: created_count, updated: updated_count })
    })
}

/// Get employees available for manual addition (active and not already in the list)
#[tauri::command]
pub async fn get_available_employees_for_current_payroll(
    pay_period_start: String,
    pay_period_end: String,
    state: State<'_, AppState>,
) -> Result<Vec<cpr_core::models::Employee>, CommandError> {
    with_db(&state, |db| {
        let period_start = NaiveDate::parse_from_str(&pay_period_start, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let period_end = NaiveDate::parse_from_str(&pay_period_end, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        let active_employees = db.employees().list_active()?;
        let mut available = Vec::new();

        for employee in active_employees {
            // Check if payroll already exists for this period in current tables
            let existing =
                db.payroll().list_by_employee(employee.id.unwrap())?.into_iter().any(|p| p.pay_period_start == period_start && p.pay_period_end == period_end);

            if !existing {
                available.push(employee);
            }
        }

        Ok(available)
    })
}

/// Check if history payroll records exist for the given dates
#[tauri::command]
pub async fn check_history_payroll_dates_exist(
    pay_period_start: String,
    pay_period_end: String,
    pay_date: String,
    state: State<'_, AppState>,
) -> Result<bool, CommandError> {
    with_db(&state, |db| {
        let period_start = NaiveDate::parse_from_str(&pay_period_start, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let period_end = NaiveDate::parse_from_str(&pay_period_end, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;
        let payment_date = NaiveDate::parse_from_str(&pay_date, "%Y-%m-%d").map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        db.payroll_history().exists_by_dates(period_start, period_end, payment_date).map_err(|e| CommandError::new(format!("Database error: {}", e)))
    })
}

/// Post current to history - updates YTD amounts and vacation accrual
#[tauri::command]
pub async fn post_current_to_history(payroll_ids: Vec<i64>, state: State<'_, AppState>) -> Result<Vec<i64>, CommandError> {
    with_db(&state, |db| {
        let mut new_ids = Vec::new();
        for id in payroll_ids {
            // Get current payroll
            let mut payroll = db.payroll().get(id)?;

            // Move to main payroll table (YTD is now automatically calculated from history)
            db.payroll_history().create(&mut payroll)?;
            let new_id = payroll.id.ok_or_else(|| CommandError::new("Failed to get new payroll ID"))?;

            new_ids.push(new_id);

            // --- Vacation Accrual (Dual: Dollar + Day) ---
            // 1. Accrue vacation earnings based on employee's vacation_pay_rate
            if let Ok(employee) = db.employees().get(payroll.employee_id) {
                // Hourly employees: accrue dollar value only (no days)
                // Non-hourly employees: accrue days only (no dollar value)
                let (accrual_amount, accrual_days) = match employee.pay_type {
                    PayType::Hourly => {
                        let dollar = calculate_vacation_accrual(payroll.gross_pay, employee.vacation_pay_rate);
                        (dollar, Decimal::ZERO)
                    }
                    _ => {
                        let total_pay_periods = payroll.total_pay_periods.max(1) as i32;
                        let days = calculate_vacation_accrual_days(employee.vacation_pay_rate, total_pay_periods);
                        (Decimal::ZERO, days)
                    }
                };

                if accrual_amount > Decimal::ZERO || accrual_days > Decimal::ZERO {
                    let balance = db.vacation().get_balance(payroll.employee_id)
                        .map_err(|e| CommandError::new(format!("Failed to get vacation balance: {}", e)))?;
                    let new_balance = balance + accrual_amount;

                    let balance_days = db.vacation().get_balance_days(payroll.employee_id)
                        .map_err(|e| CommandError::new(format!("Failed to get vacation days balance: {}", e)))?;
                    let new_balance_days = balance_days + accrual_days;

                    let mut txn = VacationAccrual::new_dual(
                        payroll.employee_id,
                        payroll.pay_date,
                        VacationTransactionType::Accrue,
                        accrual_amount,
                        new_balance,
                        accrual_days,
                        new_balance_days,
                    );
                    // Note: payroll_id left as None because the FK references the
                    // current payroll table, and this record is about to be deleted.
                    // The notes field captures the pay period for traceability.
                    txn.notes = Some(format!("Vacation accrual for pay period {}-{}", payroll.pay_period_start, payroll.pay_period_end));

                    db.vacation().record_transaction(&mut txn)
                        .map_err(|e| CommandError::new(format!("Failed to record vacation accrual: {}", e)))?;
                }

                // 2. Deduct vacation pay if the payroll includes a "vacation" earning type
                let vacation_payout: Decimal = payroll.additional_earnings.iter()
                    .filter(|e| e.earning_type.to_lowercase() == "vacation")
                    .map(|e| e.amount)
                    .sum();

                if vacation_payout > Decimal::ZERO {
                    let balance = db.vacation().get_balance(payroll.employee_id)
                        .map_err(|e| CommandError::new(format!("Failed to get vacation balance: {}", e)))?;
                    let new_balance = balance - vacation_payout;

                    // For non-hourly: also compute days equivalent of the payout
                    let days_payout = match employee.pay_type {
                        PayType::Hourly => Decimal::ZERO,
                        _ => {
                            // Days deducted = vacation_payout / (gross_pay / pay_periods / (work_days/pay_periods))
                            // Simplified: for vacation payout, the days are already tracked via time_off
                            // This is a direct dollar payout (not time off), so days stay at 0
                            Decimal::ZERO
                        }
                    };
                    let balance_days = db.vacation().get_balance_days(payroll.employee_id)
                        .map_err(|e| CommandError::new(format!("Failed to get vacation days balance: {}", e)))?;
                    let new_balance_days = balance_days - days_payout;

                    let mut txn = VacationAccrual::new_dual(
                        payroll.employee_id,
                        payroll.pay_date,
                        VacationTransactionType::Payout,
                        -vacation_payout,
                        new_balance,
                        -days_payout,
                        new_balance_days,
                    );
                    // Note: payroll_id left as None because the FK references the
                    // current payroll table, and this record is about to be deleted.
                    // The notes field captures the pay period for traceability.
                    txn.notes = Some(format!("Vacation payout for pay period {}-{}", payroll.pay_period_start, payroll.pay_period_end));

                    db.vacation().record_transaction(&mut txn)
                        .map_err(|e| CommandError::new(format!("Failed to record vacation payout: {}", e)))?;
                }
            }

            // Delete from current
            db.payroll().delete(id)?;
        }

        Ok(new_ids)
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentPayrollResult {
    pub payrolls: Vec<Payroll>,
    pub errors: Vec<CurrentPayrollError>,
    pub created: usize,
    pub updated: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CurrentPayrollError {
    pub employee_id: i64,
    pub employee_name: String,
    pub error: String,
}

/// Delete a payroll record (using update to mark as voided)
#[tauri::command]
pub async fn delete_payroll(id: i64, state: State<'_, AppState>) -> Result<(), CommandError> {
    with_db(&state, |db| {
        // Delete from current payroll (actual delete for drafts)
        db.payroll().delete(id)?;
        Ok(())
    })
}

// Current payroll helpers
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CurrentPayrollDates {
    pub pay_period_start: String,
    pub pay_period_end: String,
    pub pay_date: String,
    pub pay_period_number: Option<i32>,
    pub total_pay_periods: i32,
}

#[tauri::command]
pub async fn get_current_payroll_dates(state: State<'_, AppState>) -> Result<Option<CurrentPayrollDates>, CommandError> {
    with_db(&state, |db| {
        let dates_opt = db.payroll().get_dates()?;
        match dates_opt {
            None => Ok(None),
            Some((start, end, pay_date)) => {
                let payrolls = db.payroll().list_all()?;
                let ppn = payrolls.first().and_then(|p| p.pay_period_number);
                let tpp = payrolls.first().map_or(0, |p| p.total_pay_periods);
                Ok(Some(CurrentPayrollDates {
                    pay_period_start: start.format("%Y-%m-%d").to_string(),
                    pay_period_end: end.format("%Y-%m-%d").to_string(),
                    pay_date: pay_date.format("%Y-%m-%d").to_string(),
                    pay_period_number: ppn,
                    total_pay_periods: tpp,
                }))
            }
        }
    })
}

#[tauri::command]
pub async fn update_current_payroll(payroll: Payroll, state: State<'_, AppState>) -> Result<(), CommandError> {
    with_db(&state, |db| {
        db.payroll().update(&payroll)?;
        Ok(())
    })
}

#[tauri::command]
pub async fn list_current_payroll(state: State<'_, AppState>) -> Result<Vec<Payroll>, CommandError> {
    with_db(&state, |db| db.payroll().list_all().map_err(|e| CommandError::new(e.to_string())))
}

/// Clear all current payroll data from database
#[tauri::command]
pub async fn clear_current_payroll(state: State<'_, AppState>) -> Result<(), CommandError> {
    with_db(&state, |db| db.payroll().clear_all().map_err(|e| CommandError::new(e.to_string())))
}

/// Add a calculated payroll to current payroll table
#[tauri::command]
pub async fn add_to_current_payroll(mut payroll: Payroll, state: State<'_, AppState>) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        // Save to current payroll table
        let id = db.payroll().create(&mut payroll)?;
        Ok(id)
    })
}

/// Export current payroll to CSV file
#[tauri::command]
pub async fn export_current_payroll_csv(output_path: String, state: State<'_, AppState>) -> Result<String, CommandError> {
    with_db(&state, |db| {
        let payrolls = db.payroll().list_all()?;

        // Get all employees for reference
        let employees: std::collections::HashMap<i64, cpr_core::models::Employee> =
            db.employees().list_all()?.into_iter().filter_map(|e| e.id.map(|id| (id, e))).collect();

        // Collect all unique additional earning names across all payrolls
        let mut earning_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        // Add known earning types first
        for name in EarningType::all_names() {
            earning_names.insert(name.to_string());
        }
        for payroll in &payrolls {
            for earning in &payroll.additional_earnings {
                earning_names.insert(earning.earning_type.clone());
            }
        }

        // Collect all unique additional deduction names across all payrolls
        let mut deduction_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        // Add known deduction types first
        for name in DeductionType::all_names() {
            deduction_names.insert(name.to_string());
        }
        for payroll in &payrolls {
            for deduction in &payroll.deductions.additional {
                deduction_names.insert(deduction.name.clone());
            }
        }

        // Create CSV content
        let mut csv_content = String::new();

        // Build header row with earning/deduction names as column headers
        let mut headers = vec![
            "employee_number".to_string(),
            "first_name".to_string(),
            "last_name".to_string(),
            "pay_period_start".to_string(),
            "pay_period_end".to_string(),
            "pay_date".to_string(),
            "regular_hours".to_string(),
            "overtime_hours".to_string(),
            "gross_pay".to_string(),
        ];

        // Add each unique earning name as a column header (lowercase with underscores)
        for name in &earning_names {
            let header_name = name.to_lowercase().replace(' ', "_");
            headers.push(header_name);
        }

        // Add standard deduction columns
        headers.push("cpp".to_string());
        headers.push("cpp2".to_string());
        headers.push("ei".to_string());
        headers.push("federal_tax".to_string());
        headers.push("provincial_tax".to_string());

        // Add each unique deduction name as a column header (lowercase with underscores)
        for name in &deduction_names {
            let header_name = name.to_lowercase().replace(' ', "_");
            headers.push(header_name);
        }

        // Add remaining columns
        headers.push("net_pay".to_string());
        headers.push("pay_period_number".to_string());
        headers.push("total_pay_periods".to_string());

        csv_content.push_str(&headers.join(","));
        csv_content.push('\n');

        for payroll in &payrolls {
            let employee = employees.get(&payroll.employee_id);
            let employee_number = employee.map(|e| e.employee_number.as_str()).unwrap_or("");
            let first_name = employee.map(|e| e.first_name.as_str()).unwrap_or("");
            let last_name = employee.map(|e| e.last_name.as_str()).unwrap_or("");

            let regular_hours = payroll.regular_hours.map(|h| h.to_string()).unwrap_or_default();
            let overtime_hours = payroll.overtime_hours.map(|h| h.to_string()).unwrap_or_default();
            let pay_period_number = payroll.pay_period_number.map(|n| n.to_string()).unwrap_or_default();

            let mut fields = vec![
                employee_number.to_string(),
                first_name.to_string(),
                last_name.to_string(),
                payroll.pay_period_start.to_string(),
                payroll.pay_period_end.to_string(),
                payroll.pay_date.to_string(),
                regular_hours,
                overtime_hours,
                payroll.gross_pay.to_string(),
            ];

            // Build a map of earning name to amount for this payroll
            let earning_map: std::collections::HashMap<String, rust_decimal::Decimal> =
                payroll.additional_earnings.iter().map(|e| (e.earning_type.clone(), e.amount)).collect();

            // Add amount for each earning name column
            for name in &earning_names {
                fields.push(earning_map.get(name).map(|a| a.to_string()).unwrap_or_default());
            }

            // Add standard deduction fields
            fields.push(payroll.deductions.cpp.to_string());
            fields.push(payroll.deductions.cpp2.to_string());
            fields.push(payroll.deductions.ei.to_string());
            fields.push(payroll.deductions.federal_tax.to_string());
            fields.push(payroll.deductions.provincial_tax.to_string());

            // Build a map of deduction name to amount for this payroll
            let deduction_map: std::collections::HashMap<String, rust_decimal::Decimal> =
                payroll.deductions.additional.iter().map(|d| (d.name.clone(), d.amount)).collect();

            // Add amount for each deduction name column
            for name in &deduction_names {
                fields.push(deduction_map.get(name).map(|a| a.to_string()).unwrap_or_default());
            }

            // Add remaining fields
            fields.push(payroll.net_pay.to_string());
            fields.push(pay_period_number);
            fields.push(payroll.total_pay_periods.to_string());

            csv_content.push_str(&fields.join(","));
            csv_content.push('\n');
        }

        // Write to file
        let path = Path::new(&output_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CommandError::new(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, csv_content).map_err(|e| CommandError::new(format!("Failed to write CSV file: {}", e)))?;

        Ok(output_path)
    })
}

/// Export history payroll to CSV file with optional filters
#[tauri::command]
pub async fn export_history_payroll_csv(
    output_path: String,
    employee_id: Option<i64>,
    pay_date_from: Option<String>,
    pay_date_to: Option<String>,
    search_term: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        // Build filter
        let filter = PayrollHistoryFilter {
            employee_id,
            pay_date_from: pay_date_from.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            pay_date_to: pay_date_to.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()),
            search_term,
            limit: None, // Export all matching records
            offset: None,
        };

        let payrolls = db.payroll_history().list_filtered(&filter)?;

        // Get all employees for reference
        let employees: std::collections::HashMap<i64, cpr_core::models::Employee> =
            db.employees().list_all()?.into_iter().filter_map(|e| e.id.map(|id| (id, e))).collect();

        // Collect all unique additional earning names across all payrolls
        let mut earning_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        // Add known earning types first
        for name in EarningType::all_names() {
            earning_names.insert(name.to_string());
        }
        for payroll in &payrolls {
            for earning in &payroll.additional_earnings {
                earning_names.insert(earning.earning_type.clone());
            }
        }

        // Collect all unique additional deduction names across all payrolls
        let mut deduction_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        // Add known deduction types first
        for name in DeductionType::all_names() {
            deduction_names.insert(name.to_string());
        }
        for payroll in &payrolls {
            for deduction in &payroll.deductions.additional {
                deduction_names.insert(deduction.name.clone());
            }
        }

        // Create CSV content
        let mut csv_content = String::new();

        // Build header row with earning/deduction names as column headers
        let mut headers = vec![
            "employee_number".to_string(),
            "first_name".to_string(),
            "last_name".to_string(),
            "pay_period_start".to_string(),
            "pay_period_end".to_string(),
            "pay_date".to_string(),
            "regular_hours".to_string(),
            "overtime_hours".to_string(),
            "gross_pay".to_string(),
        ];

        // Add each unique earning name as a column header (lowercase with underscores)
        for name in &earning_names {
            let header_name = name.to_lowercase().replace(' ', "_");
            headers.push(header_name);
        }

        // Add standard deduction columns
        headers.push("cpp".to_string());
        headers.push("cpp2".to_string());
        headers.push("ei".to_string());
        headers.push("federal_tax".to_string());
        headers.push("provincial_tax".to_string());

        // Add each unique deduction name as a column header (lowercase with underscores)
        for name in &deduction_names {
            let header_name = name.to_lowercase().replace(' ', "_");
            headers.push(header_name);
        }

        // Add remaining columns
        headers.push("net_pay".to_string());
        headers.push("pay_period_number".to_string());
        headers.push("total_pay_periods".to_string());

        csv_content.push_str(&headers.join(","));
        csv_content.push('\n');

        for payroll in &payrolls {
            let employee = employees.get(&payroll.employee_id);
            let employee_number = employee.map(|e| e.employee_number.as_str()).unwrap_or("");
            let first_name = employee.map(|e| e.first_name.as_str()).unwrap_or("");
            let last_name = employee.map(|e| e.last_name.as_str()).unwrap_or("");

            let regular_hours = payroll.regular_hours.map(|h| h.to_string()).unwrap_or_default();
            let overtime_hours = payroll.overtime_hours.map(|h| h.to_string()).unwrap_or_default();
            let pay_period_number = payroll.pay_period_number.map(|n| n.to_string()).unwrap_or_default();

            let mut fields = vec![
                employee_number.to_string(),
                first_name.to_string(),
                last_name.to_string(),
                payroll.pay_period_start.to_string(),
                payroll.pay_period_end.to_string(),
                payroll.pay_date.to_string(),
                regular_hours,
                overtime_hours,
                payroll.gross_pay.to_string(),
            ];

            // Build a map of earning name to amount for this payroll
            let earning_map: std::collections::HashMap<String, rust_decimal::Decimal> =
                payroll.additional_earnings.iter().map(|e| (e.earning_type.clone(), e.amount)).collect();

            // Add amount for each earning name column
            for name in &earning_names {
                fields.push(earning_map.get(name).map(|a| a.to_string()).unwrap_or_default());
            }

            // Add standard deduction fields
            fields.push(payroll.deductions.cpp.to_string());
            fields.push(payroll.deductions.cpp2.to_string());
            fields.push(payroll.deductions.ei.to_string());
            fields.push(payroll.deductions.federal_tax.to_string());
            fields.push(payroll.deductions.provincial_tax.to_string());

            // Build a map of deduction name to amount for this payroll
            let deduction_map: std::collections::HashMap<String, rust_decimal::Decimal> =
                payroll.deductions.additional.iter().map(|d| (d.name.clone(), d.amount)).collect();

            // Add amount for each deduction name column
            for name in &deduction_names {
                fields.push(deduction_map.get(name).map(|a| a.to_string()).unwrap_or_default());
            }

            // Add remaining fields
            fields.push(payroll.net_pay.to_string());
            fields.push(pay_period_number);
            fields.push(payroll.total_pay_periods.to_string());

            csv_content.push_str(&fields.join(","));
            csv_content.push('\n');
        }

        // Write to file
        let path = Path::new(&output_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CommandError::new(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(path, csv_content).map_err(|e| CommandError::new(format!("Failed to write CSV file: {}", e)))?;

        Ok(output_path)
    })
}

/// Import current payroll from CSV file
/// Fields are optional - only provided fields will be used
#[tauri::command]
pub async fn import_current_payroll_csv(
    file_path: String,
    pay_period_start: Option<String>,
    pay_period_end: Option<String>,
    pay_date: Option<String>,
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CurrentPayrollResult, CommandError> {
    with_db(&state, |db| {
        // Read CSV file
        let csv_content = fs::read_to_string(&file_path).map_err(|e| CommandError::new(format!("Failed to read CSV file: {}", e)))?;

        let mut lines = csv_content.lines();

        // Skip header
        let header = lines.next().ok_or_else(|| CommandError::new("CSV file is empty"))?;

        // Parse header to find column indices
        // Accepted columns: employee_number (required), regular_hours, overtime_hours, gross_pay,
        // allowance, benefit, bonus, commission, other, vacation, net_pay_adjust, group_insurance,
        // addon_tax, pension_rrsp, union_dues
        let headers: Vec<&str> = header.split(',').collect();
        let get_index = |name: &str| -> Option<usize> { headers.iter().position(|&h| h.trim().to_lowercase() == name.to_lowercase()) };

        // Required column
        let idx_employee_number = get_index("employee_number");

        // Optional earnings columns
        let idx_regular_hours = get_index("regular_hours");
        let idx_overtime_hours = get_index("overtime_hours");
        let idx_gross_pay = get_index("gross_pay");

        // Optional metadata columns
        let idx_pay_period_number = get_index("pay_period_number");
        let idx_total_pay_periods = get_index("total_pay_periods");

        // Get all employees for matching
        let employees: Vec<cpr_core::models::Employee> = db.employees().list_all()?;
        let employee_by_number: std::collections::HashMap<String, &cpr_core::models::Employee> =
            employees.iter().filter_map(|e| Some((e.employee_number.clone(), e))).collect();

        let mut payrolls = Vec::new();
        let mut errors = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;

        for (line_num, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();

            // Helper to get field value
            let get_field = |idx: Option<usize>| -> Option<&str> { idx.and_then(|i| fields.get(i)).map(|s| s.trim()) };

            // Get employee by employee_number (required)
            let employee = if let Some(emp_number) = get_field(idx_employee_number) { employee_by_number.get(emp_number).copied() } else { None };

            let employee = match employee {
                Some(e) => e,
                None => {
                    errors.push(CurrentPayrollError {
                        employee_id: 0,
                        employee_name: get_field(idx_employee_number).unwrap_or("Unknown").to_string(),
                        error: format!("Line {}: Employee not found", line_num + 2),
                    });
                    continue;
                }
            };

            // Parse dates from function parameters (dates are defined during payroll creation, not in CSV)
            let period_start = pay_period_start.as_ref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let period_end = pay_period_end.as_ref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let payment_date = pay_date.as_ref().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

            // Dates are required for tax calculations - skip row if not available
            let (period_start, period_end, payment_date) = match (period_start, period_end, payment_date) {
                (Some(s), Some(e), Some(p)) => (s, e, p),
                _ => {
                    // Skip rows without complete date information
                    continue;
                }
            };

            // Parse numeric fields
            let regular_hours = get_field(idx_regular_hours).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() });
            let overtime_hours = get_field(idx_overtime_hours).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() });
            let gross_pay = get_field(idx_gross_pay).and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO);

            // Parse additional earnings from accepted columns using EarningType enum
            let mut additional_earnings = Vec::new();

            for earning_type in cpr_core::models::payroll::EarningType::all_names() {
                let idx = get_index(earning_type);
                if let Some(value) = get_field(idx) {
                    if !value.is_empty() {
                        if let Ok(amount) = Decimal::from_str(value) {
                            if amount != Decimal::ZERO {
                                // Parse the earning type from string to get the enum variant
                                let earning_enum = if let Some(e) = cpr_core::models::payroll::EarningType::from_str(earning_type) {
                                    e
                                } else {
                                    errors.push(CurrentPayrollError {
                                        employee_id: 0,
                                        employee_name: get_field(idx_employee_number).unwrap_or("Unknown").to_string(),
                                        error: format!("Line {}: Invalid earning type '{}'", line_num + 2, earning_type),
                                    });
                                    continue;
                                };

                                additional_earnings.push(cpr_core::models::payroll::AdditionalEarning {
                                    id: None,
                                    payroll_id: 0, // Will be set when saved
                                    earning_type: earning_enum.as_str().to_string(),
                                    amount,
                                    hours: None,
                                    is_periodic: earning_enum.is_periodic(),
                                });
                            }
                        }
                    }
                }
            }

            let additional_earnings_total: Decimal = additional_earnings.iter().map(|e| e.amount).sum();

            // Calculate non-periodic earnings total (bonus, commission, retroactive, accumulated_overtime)
            let non_periodic_earnings: Decimal = additional_earnings.iter().filter(|e| !e.is_periodic).map(|e| e.amount).sum();

            // Parse additional deductions from accepted columns using DeductionType enum
            let mut additional_deductions_vec = Vec::new();

            for deduction_type in cpr_core::models::payroll::DeductionType::all_names() {
                let idx = get_index(deduction_type);
                if let Some(value) = get_field(idx) {
                    if !value.is_empty() {
                        if let Ok(amount) = Decimal::from_str(value) {
                            if amount != Decimal::ZERO {
                                // Parse the deduction type from string to get the enum variant
                                let deduction_enum = if let Some(e) = cpr_core::models::payroll::DeductionType::from_str(deduction_type) {
                                    e
                                } else {
                                    errors.push(CurrentPayrollError {
                                        employee_id: 0,
                                        employee_name: get_field(idx_employee_number).unwrap_or("Unknown").to_string(),
                                        error: format!("Line {}: Invalid deduction type '{}'", line_num + 2, deduction_type),
                                    });
                                    continue;
                                };

                                additional_deductions_vec
                                    .push(cpr_core::models::payroll::AdditionalDeduction { name: deduction_enum.as_str().to_string(), amount });
                            }
                        }
                    }
                }
            }

            let additional_deductions: Decimal = additional_deductions_vec.iter().map(|d| d.amount).sum();

            let pay_period_number = get_field(idx_pay_period_number).and_then(|s| if s.is_empty() { None } else { s.parse::<i32>().ok() });

            let total_pay_periods = get_field(idx_total_pay_periods).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);

            // Calculate pay period number if not provided
            let pay_period_number = pay_period_number.or_else(|| {
                let period_duration = period_end - period_start;
                let period_days = period_duration.num_days() + 1;
                let year_start = NaiveDate::from_ymd_opt(payment_date.year(), 1, 1)?;
                let days_since_year_start = period_start.signed_duration_since(year_start).num_days();
                Some(std::cmp::max((days_since_year_start / period_days) as i32 + 1, 1))
            });

            // Calculate deductions using tax calculator
            let period_duration = period_end - period_start;
            let period_days_i64 = period_duration.num_days() + 1;
            let pay_periods_per_year = get_pay_periods_per_year(period_days_i64);

            // Calculate taxable income (gross pay + additional earnings)
            let total_gross = gross_pay + additional_earnings_total;

            // Create T4127 context for the year
            let mut tax_ctx = T4127Context::new(payment_date.year())
                .map_err(|e| CommandError::new(format!("Failed to load tax config for year {} : {}", payment_date.year(), e.to_string())))?;

            // Calculate all deductions using unified T4127 context
            // This ensures CPP, EI, and taxes are calculated together with proper interdependencies
            // The method retrieves YTD values and employee info internally using the providers
            let ytd_provider = DbYtdProvider::new(db);
            let employee_provider = DbEmployeeProvider::new(db);
            let employee_info = employee_provider.get_employee_info(employee.id.unwrap(), payment_date.year())?;
            let payroll_deductions = tax_ctx.calculate_all_deductions(
                employee.id.unwrap(),
                total_gross,
                non_periodic_earnings,
                pay_periods_per_year,
                payment_date,
                &ytd_provider,
                &employee_provider,
                &additional_deductions_vec,
            )?;

            let deductions = Deductions {
                cpp: payroll_deductions.cpp,
                cpp2: payroll_deductions.cpp2,
                ei: payroll_deductions.ei,
                federal_tax: payroll_deductions.federal_tax,
                provincial_tax: payroll_deductions.provincial_tax,
                additional: additional_deductions_vec,
            };

            let total_deductions = deductions.total();
            let net_pay = total_gross - total_deductions;

            // Check if payroll already exists for this employee in current table (only one per employee)
            let existing_payrolls = db.payroll().list_by_employee(employee.id.unwrap())?;
            let existing_payroll = if existing_payrolls.is_empty() {
                None
            } else {
                // Take the first/only existing payroll for this employee
                Some(existing_payrolls[0].clone())
            };

            let mut payroll = Payroll {
                id: None,
                employee_id: employee.id.unwrap(),
                pay_period_start: period_start,
                pay_period_end: period_end,
                pay_date: payment_date,
                regular_hours,
                overtime_hours,
                additional_earnings,
                insured_earning: gross_pay,
                gross_pay,
                additional_earnings_total,
                additional_tax_amount: Decimal::ZERO,
                deductions,
                net_pay,
                pay_period_number,
                total_pay_periods,
                total_deductions,
                additional_deductions,
                federal_personal_amount: employee_info.federal_personal_amount,
                province: employee_info.province.code().to_string(),
                provincial_personal_amount: employee_info.provincial_personal_amount,
                remittance_id: None,
                created_at: Utc::now(),
            };

            if let Some(existing) = existing_payroll {
                // Update existing payroll: preserve ID and created_at
                payroll.id = existing.id;
                payroll.created_at = existing.created_at;
                match db.payroll().update(&mut payroll) {
                    Ok(_) => {
                        payrolls.push(payroll);
                        updated_count += 1;
                    }
                    Err(e) => {
                        errors.push(CurrentPayrollError {
                            employee_id: employee.id.unwrap(),
                            employee_name: format!("{} {}", employee.first_name, employee.last_name),
                            error: format!("Line {}: Failed to update: {}", line_num + 2, e),
                        });
                    }
                }
            } else {
                // Create new payroll
                match db.payroll().create(&mut payroll) {
                    Ok(_) => {
                        payrolls.push(payroll);
                        created_count += 1;
                    }
                    Err(e) => {
                        errors.push(CurrentPayrollError {
                            employee_id: employee.id.unwrap(),
                            employee_name: format!("{} {}", employee.first_name, employee.last_name),
                            error: format!("Line {}: Failed to save: {}", line_num + 2, e),
                        });
                    }
                }
            }
        }

        Ok(CurrentPayrollResult { payrolls, errors, created: created_count, updated: updated_count })
    })
}

/// Result type for history payroll import
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoryPayrollImportResult {
    pub imported: usize,
    pub updated: usize,
    pub errors: Vec<String>,
}

/// Import history payroll from CSV file
/// This accepts all data including deductions without any calculation
/// Format matches the export format: employee_number, dates are required, all others optional
#[tauri::command]
pub async fn import_history_payroll_csv(file_path: String, state: State<'_, AppState>) -> Result<HistoryPayrollImportResult, CommandError> {
    with_db(&state, |db| {
        // Read CSV file
        let csv_content = fs::read_to_string(&file_path).map_err(|e| CommandError::new(format!("Failed to read CSV file: {}", e)))?;

        let mut lines = csv_content.lines();

        // Parse header to find column indices
        let header = lines.next().ok_or_else(|| CommandError::new("CSV file is empty"))?;

        let headers: Vec<&str> = header.split(',').collect();
        let get_index = |name: &str| -> Option<usize> { headers.iter().position(|&h| h.trim().to_lowercase() == name.to_lowercase()) };

        // Required columns
        let idx_employee_number = get_index("employee_number");
        let idx_pay_period_start = get_index("pay_period_start");
        let idx_pay_period_end = get_index("pay_period_end");
        let idx_pay_date = get_index("pay_date");

        // Optional standard columns
        let idx_regular_hours = get_index("regular_hours");
        let idx_overtime_hours = get_index("overtime_hours");
        let idx_gross_pay = get_index("gross_pay");
        let idx_cpp = get_index("cpp");
        let idx_cpp2 = get_index("cpp2");
        let idx_ei = get_index("ei");
        let idx_federal_tax = get_index("federal_tax");
        let idx_provincial_tax = get_index("provincial_tax");
        let idx_net_pay = get_index("net_pay");
        let idx_pay_period_number = get_index("pay_period_number");
        let idx_total_pay_periods = get_index("total_pay_periods");

        // Identify additional earnings columns (not standard columns)
        let standard_columns = [
            "employee_number",
            "first_name",
            "last_name",
            "pay_period_start",
            "pay_period_end",
            "pay_date",
            "regular_hours",
            "overtime_hours",
            "gross_pay",
            "cpp",
            "cpp2",
            "ei",
            "federal_tax",
            "provincial_tax",
            "net_pay",
            "pay_period_number",
            "total_pay_periods",
        ];

        let mut additional_earning_columns: Vec<(usize, String)> = Vec::new();
        let mut additional_deduction_columns: Vec<(usize, String)> = Vec::new();

        for (idx, header) in headers.iter().enumerate() {
            let header_lower = header.trim().to_lowercase();
            if !standard_columns.contains(&header_lower.as_str()) && !header_lower.is_empty() {
                let header_trimmed = header.trim();
                if EarningType::from_str(header_trimmed).is_some() {
                    additional_earning_columns.push((idx, header_trimmed.to_string()));
                } else if DeductionType::from_str(header_trimmed).is_some() {
                    additional_deduction_columns.push((idx, header_trimmed.to_string()));
                } else {
                    // Unknown column - treat as deduction by default
                    additional_deduction_columns.push((idx, header_trimmed.to_string()));
                }
            }
        }

        // Get all employees for matching
        let employees: Vec<cpr_core::models::Employee> = db.employees().list_all()?;
        let employee_by_number: std::collections::HashMap<String, &cpr_core::models::Employee> =
            employees.iter().filter_map(|e| Some((e.employee_number.clone(), e))).collect();

        let mut imported_count = 0;
        let mut updated_count = 0;
        let mut errors = Vec::new();

        for (line_num, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line.split(',').collect();

            // Helper to get field value
            let get_field = |idx: Option<usize>| -> Option<&str> { idx.and_then(|i| fields.get(i)).map(|s| s.trim()) };

            // Get employee by employee_number (required)
            let employee_number = get_field(idx_employee_number);
            let employee = match employee_number {
                Some(emp_num) if !emp_num.is_empty() => match employee_by_number.get(emp_num) {
                    Some(e) => *e,
                    None => {
                        errors.push(format!("Line {}: Employee '{}' not found", line_num + 2, emp_num));
                        continue;
                    }
                },
                _ => {
                    errors.push(format!("Line {}: Missing employee_number", line_num + 2));
                    continue;
                }
            };

            // Parse required dates
            let period_start = get_field(idx_pay_period_start).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let period_end = get_field(idx_pay_period_end).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
            let payment_date = get_field(idx_pay_date).and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

            let (period_start, period_end, payment_date) = match (period_start, period_end, payment_date) {
                (Some(s), Some(e), Some(p)) => (s, e, p),
                _ => {
                    errors.push(format!("Line {}: Missing or invalid date fields", line_num + 2));
                    continue;
                }
            };

            // Parse optional numeric fields (default to 0 if not provided)
            let gross_pay = get_field(idx_gross_pay).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let cpp = get_field(idx_cpp).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let cpp2 = get_field(idx_cpp2).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let ei = get_field(idx_ei).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let federal_tax = get_field(idx_federal_tax).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let provincial_tax =
                get_field(idx_provincial_tax).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let net_pay = get_field(idx_net_pay).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() }).unwrap_or(Decimal::ZERO);

            let regular_hours = get_field(idx_regular_hours).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() });

            let overtime_hours = get_field(idx_overtime_hours).and_then(|s| if s.is_empty() { None } else { Decimal::from_str(s).ok() });

            let pay_period_number = get_field(idx_pay_period_number).and_then(|s| if s.is_empty() { None } else { s.parse::<i32>().ok() });

            let total_pay_periods = get_field(idx_total_pay_periods).and_then(|s| if s.is_empty() { None } else { s.parse::<i32>().ok() }).unwrap_or(0);

            // Calculate pay period number if not provided
            let pay_period_number = pay_period_number.or_else(|| {
                let period_duration = period_end - period_start;
                let period_days = period_duration.num_days() + 1;
                let year_start = NaiveDate::from_ymd_opt(payment_date.year(), 1, 1)?;
                let days_since_year_start = period_start.signed_duration_since(year_start).num_days();
                Some(std::cmp::max((days_since_year_start / period_days) as i32 + 1, 1))
            });

            // Parse additional earnings
            let mut additional_earnings: Vec<cpr_core::models::payroll::AdditionalEarning> = Vec::new();
            for (idx, earning_type) in &additional_earning_columns {
                if let Some(amount_str) = fields.get(*idx) {
                    let amount_str = amount_str.trim();
                    if !amount_str.is_empty() {
                        if let Ok(amount) = Decimal::from_str(amount_str) {
                            if amount != Decimal::ZERO {
                                let is_periodic = cpr_core::models::payroll::AdditionalEarning::is_periodic_earning_type(earning_type);
                                additional_earnings.push(cpr_core::models::payroll::AdditionalEarning {
                                    id: None,
                                    payroll_id: 0, // Will be set after insert
                                    earning_type: earning_type.clone(),
                                    amount,
                                    hours: None,
                                    is_periodic,
                                });
                            }
                        }
                    }
                }
            }

            // Parse additional deductions
            let mut additional_deductions: Vec<cpr_core::models::payroll::AdditionalDeduction> = Vec::new();
            for (idx, deduction_name) in &additional_deduction_columns {
                if let Some(amount_str) = fields.get(*idx) {
                    let amount_str = amount_str.trim();
                    if !amount_str.is_empty() {
                        if let Ok(amount) = Decimal::from_str(amount_str) {
                            if amount != Decimal::ZERO {
                                additional_deductions.push(cpr_core::models::payroll::AdditionalDeduction { name: deduction_name.clone(), amount });
                            }
                        }
                    }
                }
            }

            let additional_earnings_total: Decimal = additional_earnings.iter().map(|e| e.amount).sum();
            let additional_deductions_total: Decimal = additional_deductions.iter().map(|d| d.amount).sum();

            // Get employee info for province and personal amounts
            let employee_provider = DbEmployeeProvider::new(db);
            let employee_info = employee_provider.get_employee_info(employee.id.unwrap(), payment_date.year())?;

            let deductions = Deductions { cpp, cpp2, ei, federal_tax, provincial_tax, additional: additional_deductions };

            let total_deductions = deductions.total();

            // Check if history payroll exists for this employee + dates combination
            let existing_payroll = db.payroll_history()
                .list_by_employee(employee.id.unwrap())?
                .into_iter()
                .find(|p| p.pay_period_start == period_start
                        && p.pay_period_end == period_end
                        && p.pay_date == payment_date);

            let mut payroll = Payroll {
                id: None,
                employee_id: employee.id.unwrap(),
                pay_period_start: period_start,
                pay_period_end: period_end,
                pay_date: payment_date,
                regular_hours,
                overtime_hours,
                additional_earnings,
                insured_earning: gross_pay,
                gross_pay,
                additional_earnings_total,
                additional_tax_amount: Decimal::ZERO,
                deductions,
                net_pay,
                pay_period_number,
                total_pay_periods,
                total_deductions,
                additional_deductions: additional_deductions_total,
                federal_personal_amount: employee_info.federal_personal_amount,
                province: employee_info.province.code().to_string(),
                provincial_personal_amount: employee_info.provincial_personal_amount,
                remittance_id: None,
                created_at: Utc::now(),
            };

            if let Some(existing) = existing_payroll {
                // Update existing
                payroll.id = existing.id;
                payroll.created_at = existing.created_at;
                match db.payroll_history().update(&mut payroll) {
                    Ok(_) => {
                        updated_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("Line {}: Failed to update: {}", line_num + 2, e));
                    }
                }
            } else {
                // Create new
                match db.payroll_history().create_import(&mut payroll) {
                    Ok(_) => imported_count += 1,
                    Err(e) => {
                        errors.push(format!("Line {}: Failed to save: {}", line_num + 2, e));
                    }
                }
            }
        }

        Ok(HistoryPayrollImportResult { imported: imported_count, updated: updated_count, errors })
    })
}

/// Delete a history payroll record by ID
#[tauri::command]
pub async fn delete_history_payroll(id: i64, state: State<'_, AppState>) -> Result<(), CommandError> {
    with_db(&state, |db| db.payroll_history().delete(id).map_err(|e| CommandError::new(format!("Failed to delete history payroll: {}", e))))
}

/// Update a history payroll record
#[tauri::command]
pub async fn update_history_payroll(payroll: Payroll, state: State<'_, AppState>) -> Result<(), CommandError> {
    with_db(&state, |db| db.payroll_history().update(&payroll).map_err(|e| CommandError::new(format!("Failed to update history payroll: {}", e))))
}

/// Rust representation of PayrollPeriod for Tauri command responses
/// Uses String for dates to avoid serialization issues with NaiveDate
#[derive(Debug, serde::Serialize)]
pub struct PayrollPeriodRust {
    pub pay_period_start: String,
    pub pay_period_end: String,
    pub pay_date: String,
}

/// List distinct years that have payroll history records
#[tauri::command]
pub async fn list_payroll_years(employee_id: Option<i64>, state: State<'_, AppState>) -> Result<Vec<i32>, CommandError> {
    with_db(&state, |db| db.payroll_history().list_years(employee_id).map_err(|e| CommandError::new(format!("Failed to list payroll years: {}", e))))
}

/// List distinct pay periods for a given year
#[tauri::command]
pub async fn list_payroll_periods(year: i32, employee_id: Option<i64>, state: State<'_, AppState>) -> Result<Vec<PayrollPeriodRust>, CommandError> {
    with_db(&state, |db| {
        let periods = db.payroll_history().list_pay_periods(year, employee_id).map_err(|e| CommandError::new(format!("Failed to list pay periods: {}", e)))?;
        Ok(periods.into_iter().map(|p| PayrollPeriodRust {
            pay_period_start: p.pay_period_start.format("%Y-%m-%d").to_string(),
            pay_period_end: p.pay_period_end.format("%Y-%m-%d").to_string(),
            pay_date: p.pay_date.format("%Y-%m-%d").to_string(),
        }).collect())
    })
}

#[derive(serde::Serialize)]
#[allow(dead_code)]
pub struct AdditionalTypesResponse {
    pub earnings: Vec<EarningTypeInfo>,
    pub deductions: Vec<DeductionTypeInfo>,
}

#[derive(serde::Serialize)]
#[allow(dead_code)]
pub struct EarningTypeInfo {
    pub name: String,
    pub display_name: String,
    pub is_periodic: bool,
}

#[derive(serde::Serialize)]
#[allow(dead_code)]
pub struct DeductionTypeInfo {
    pub name: String,
    pub display_name: String,
    pub t4127_variable: Option<String>,
}

#[tauri::command]
#[allow(dead_code)]
pub fn get_additional_types() -> AdditionalTypesResponse {
    AdditionalTypesResponse {
        earnings: EarningType::all_names()
            .iter()
            .map(|name| {
                let et = EarningType::from_str(name).unwrap();
                EarningTypeInfo { name: name.to_string(), display_name: et.display_name().to_string(), is_periodic: et.is_periodic() }
            })
            .collect(),
        deductions: DeductionType::all_names()
            .iter()
            .map(|name| {
                let dt = DeductionType::from_str(name).unwrap();
                DeductionTypeInfo {
                    name: name.to_string(),
                    display_name: dt.display_name().to_string(),
                    t4127_variable: dt.t4127_variable().map(|s| s.to_string()),
                }
            })
            .collect(),
    }
}

/// Save a raw history payroll record without validation (for developer mode)
#[tauri::command]
pub async fn save_raw_payroll(mut payroll: Payroll, state: State<'_, AppState>) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let id = db.payroll_history().create_import(&mut payroll).map_err(|e| CommandError::new(format!("Failed to save raw payroll: {}", e)))?;
        Ok(id)
    })
}
