use tauri::{State, AppHandle};
use crate::state::AppState;
use crate::error::CommandError;
use cpr_core::models::{Employee, PayRateHistory, EmploymentHistory, PersonalAmount, Province, PayType};
use cpr_core::tax::load_tax_config;

use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

/// Helper to get database from state
fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&cpr_db::repository::Database) -> Result<T, CommandError>,
{
    let db_guard = state.database.lock().unwrap();
    let db = db_guard.as_ref()
        .ok_or_else(|| CommandError::new("No database open"))?;
    f(db)
}

/// List all employees
#[tauri::command]
pub async fn list_employees(
    state: State<'_, AppState>,
) -> Result<Vec<Employee>, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_all()?;
        Ok(employees)
    })
}

/// Get a single employee by ID
#[tauri::command]
pub async fn get_employee(
    id: i64,
    state: State<'_, AppState>,
) -> Result<Employee, CommandError> {
    with_db(&state, |db| {
        let employee = db.employees().get(id)?;
        Ok(employee)
    })
}

/// Create a new employee
#[tauri::command]
pub async fn create_employee(
    mut employee: Employee,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        // Get company information to determine province for personal amounts
        let company = db.company().get()
            .map_err(|e| CommandError::new(&format!("Failed to get company: {}", e)))?
            .ok_or_else(|| CommandError::new("No company configured"))?;
        let company_province = company.province;
        
        let id = db.employees().create(&mut employee)?;
        let today = Utc::now().date_naive();
        let mut pay_history = PayRateHistory::new(
            id,
            employee.pay_rate.clone(),
            employee.pay_type,
            today,
            Some("Initial pay rate on hire".to_string()),
        );
        db.history().add_pay_rate(&mut pay_history)?;
        let mut emp_history = EmploymentHistory::new(
            id,
            employee.hire_date,
        );
        db.history().add_employment(&mut emp_history)?;
        
        // Create default personal amount with company province
        let _ = create_default_personal_amount(db, id, &company_province);

        Ok(id)
    })
}

/// Update an existing employee
#[tauri::command]
pub async fn update_employee(
    employee: Employee,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let id = employee.id.ok_or_else(|| CommandError::new("Employee ID required"))?;
        let old_employee = db.employees().get(id)?;
        // Track pay rate changes
        if old_employee.pay_rate != employee.pay_rate || old_employee.pay_type != employee.pay_type {
            let today = Utc::now().date_naive();
            db.history().close_pay_rate_history(id, today)?;
            let mut new_history = PayRateHistory::new(
                id,
                employee.pay_rate.clone(),
                employee.pay_type,
                today,
                Some("Pay rate updated via employee form".to_string()),
            );
            db.history().add_pay_rate(&mut new_history)?;
        }
        // Track employment changes
        if old_employee.hire_date != employee.hire_date || old_employee.termination_date != employee.termination_date {
            if let Some(mut curr_eh) = db.history().get_current_employment(id)? {
                curr_eh.hire_date = employee.hire_date;
                if let Some(term_date) = employee.termination_date {
                    curr_eh.terminate(term_date, Some("Employment updated via employee form".to_string()), employee.is_active);
                } else {
                    curr_eh.termination_date = None;
                    curr_eh.termination_reason = None;
                    curr_eh.rehire_eligible = true;
                }
                db.history().update_employment(&curr_eh)?;
            } else {
                let mut new_eh = EmploymentHistory::new(id, employee.hire_date);
                if let Some(term_date) = employee.termination_date {
                    new_eh.terminate(term_date, Some("Employment updated via employee form".to_string()), employee.is_active);
                }
                db.history().add_employment(&mut new_eh)?;
            }
        }
        db.employees().update(&employee)?;

        Ok(())
    })
}

/// Delete an employee
#[tauri::command]
pub async fn delete_employee(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        db.employees().delete(id)?;
        Ok(())
    })
}

/// Search employees by employee number
#[tauri::command]
pub async fn search_employees(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Employee>, CommandError> {
    with_db(&state, |db| {
        // For now, filter by employee number using find_by_number
        // A full-text search can be added later
        let result = db.employees().find_by_number(&query)?;
        Ok(result.into_iter().collect())
    })
}

/// Get pay rate history for an employee
#[tauri::command]
pub async fn get_pay_rate_history(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PayRateHistory>, CommandError> {
    with_db(&state, |db| {
        let history = db.history().get_pay_rate_history(employee_id)?;
        Ok(history)
    })
}

/// Get employment history for an employee
#[tauri::command]
pub async fn get_employment_history(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<EmploymentHistory>, CommandError> {
    with_db(&state, |db| {
        let history = db.history().get_employment_history(employee_id)?;
        Ok(history)
    })
}

/// Get list of active employees
#[tauri::command]
pub async fn list_active_employees(
    state: State<'_, AppState>,
) -> Result<Vec<Employee>, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_active()?;
        Ok(employees)
    })
}

/// Get personal amount for specific employee, province and year
#[tauri::command]
pub async fn get_personal_amount(
    employee_id: i64,
    province: String,
    year: i32,
    state: State<'_, AppState>,
) -> Result<PersonalAmount, CommandError> {
    with_db(&state, |db| Ok(
        db.personal_amount()
            .get_by_employee_province_year(employee_id, &province, year)
            .map_err(CommandError::from)?
            .ok_or_else(|| CommandError::new(format!("No personal amount found for employee {} in {} for {}", employee_id, province, year)))?
    ))
}

/// Get all personal amounts for an employee
#[tauri::command]
pub async fn get_personal_amounts(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PersonalAmount>, CommandError> {
    with_db(&state, |db| Ok(
        db.personal_amount().list_by_employee(employee_id).map_err(CommandError::from)?
    ))
}

/// Get the latest personal amount for an employee by province
#[tauri::command]
pub async fn get_latest_personal_amount_by_province(
    employee_id: i64,
    province: String,
    state: State<'_, AppState>,
) -> Result<Option<PersonalAmount>, CommandError> {
    with_db(&state, |db| {
        let result = db.personal_amount()
            .get_latest_by_employee_and_province(employee_id, &province)
            .map_err(CommandError::from)?;
        Ok(result)
    })
}

/// Create a new personal amount
#[tauri::command]
pub async fn create_personal_amount(
    mut personal_amount: PersonalAmount,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| Ok(
        db.personal_amount().create(&mut personal_amount).map_err(CommandError::from)?
    ))
}

/// Update an existing personal amount
#[tauri::command]
pub async fn update_personal_amount(
    personal_amount: PersonalAmount,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let id = personal_amount.id.ok_or_else(|| CommandError::new("PersonalAmount ID required"))?;
        db.personal_amount().update(id, &personal_amount).map_err(CommandError::from)?;
        Ok(())
    })
}

/// Resolve the config directory path, checking multiple locations
/// Handles both dev mode and production builds (including _up_ resource paths)
fn resolve_config_dir() -> Option<std::path::PathBuf> {
    let candidates = [
        std::path::PathBuf::from("config"),
        std::path::PathBuf::from("../config"),
        std::path::PathBuf::from("../../config"),
        std::path::PathBuf::from("_up_/config"),
        std::path::PathBuf::from("../_up_/config"),
    ];
    
    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

/// Resolve the config directory path using Tauri's resource_dir (for commands with AppHandle)
fn resolve_config_dir_with_app(app: &AppHandle) -> Option<std::path::PathBuf> {
    if let Some(resource_dir) = app.path_resolver().resource_dir() {
        // Try direct path first (correct with tuple resource mapping)
        let direct = resource_dir.join("config");
        if direct.exists() {
            return Some(direct);
        }
        // Fallback: _up_ path (when resources use ../ prefix without tuple mapping)
        let up_path = resource_dir.join("_up_/config");
        if up_path.exists() {
            return Some(up_path);
        }
    }
    // Fallback to filesystem paths (dev mode)
    resolve_config_dir()
}

/// Get available tax years from config files
#[tauri::command]
pub async fn get_available_tax_years(app: AppHandle) -> Result<Vec<i32>, CommandError> {
    use std::collections::HashSet;
    use std::fs;
    
    let mut years = HashSet::new();
    
    let config_dir = resolve_config_dir_with_app(&app)
        .ok_or_else(|| CommandError::new(
            "Config directory not found. Please ensure the config/ directory exists."
        ))?;
    
    let entries = fs::read_dir(&config_dir)
        .map_err(|e| CommandError::new(&format!("Failed to read config directory at {:?}: {}", config_dir, e)))?;
    
    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            // Match filenames like "tax_rates_2024.json" or "tax_rates_2025_jul.json"
            if filename.starts_with("tax_rates_") && filename.ends_with(".json") {
                // Extract year from filename
                let without_prefix = filename.strip_prefix("tax_rates_").unwrap();
                let year_part = without_prefix.split('_').next()
                    .or_else(|| without_prefix.strip_suffix(".json"));
                
                if let Some(year_str) = year_part {
                    let year_str = year_str.trim_end_matches(".json");
                    if let Ok(year) = year_str.parse::<i32>() {
                        years.insert(year);
                    }
                }
            }
        }
    }
    
    // Return error if no config files found
    if years.is_empty() {
        return Err(CommandError::new(
            "No tax configuration files found in config directory. Please ensure tax_rates_*.json files exist."
        ));
    }
    
    // Convert to sorted Vec (descending order for UI)
    let mut years_vec: Vec<i32> = years.into_iter().collect();
    years_vec.sort_by(|a, b| b.cmp(a)); // Sort descending (2026, 2025, 2024)
    
    Ok(years_vec)
}

/// Structure for returning basic personal amounts from tax config
#[derive(Debug, Serialize)]
pub struct BasicPersonalAmounts {
    pub federal_amount: f64,
    pub provincial_amount: f64,
}

/// Get basic personal amounts from tax configuration for a given province and year
#[tauri::command]
pub async fn get_basic_personal_amounts(
    province: Province,
    year: i32,
    _app: AppHandle,
) -> Result<BasicPersonalAmounts, CommandError> {
    let tax_config = load_tax_config(year)
        .map_err(|e| CommandError::new(format!("Failed to load tax config for year {}: {}", year, e.to_string())))?;

    let federal_amount = tax_config.federal.basic_personal_amount;
    
    let provincial_config = tax_config.provincial.province_configs
        .get(&province)
        .ok_or_else(|| CommandError::new(&format!("No tax configuration found for province: {:?}", province)))?;
    
    let provincial_amount = provincial_config.basic_personal_amount;
    
    Ok(BasicPersonalAmounts {
        federal_amount: federal_amount.to_string().parse().unwrap_or(0.0),
        provincial_amount: provincial_amount.to_string().parse().unwrap_or(0.0),
    })
}

/// Structure for returning tax rates needed for T4 validation
#[derive(Debug, Serialize)]
pub struct TaxRatesForValidation {
    /// CPP employee rate (e.g., 0.0595 for 2025)
    pub cpp_employee_rate: f64,
    /// CPP basic exemption ($3,500/year)
    pub cpp_basic_exemption: f64,
    /// CPP year's maximum pensionable earnings (YMPE)
    pub cpp_ympe: f64,
    /// CPP maximum contribution
    pub cpp_max_contribution: f64,
    /// CPP2 employee rate (e.g., 0.063 for 2025/2026)
    pub cpp2_rate: f64,
    /// CPP2 maximum earnings
    pub cpp2_max_earnings: f64,
    /// CPP2 maximum contribution
    pub cpp2_max_contribution: f64,
    /// EI employee rate (non-Quebec)
    pub ei_rate: f64,
    /// EI maximum insurable earnings
    pub ei_max_insurable_earnings: f64,
    /// EI maximum employee premium
    pub ei_max_contribution: f64,
}

/// Get tax rates for T4 validation for a given year
#[tauri::command]
pub async fn get_tax_rates(
    year: i32,
) -> Result<TaxRatesForValidation, CommandError> {
    let tax_config = load_tax_config(year)
        .map_err(|e| CommandError::new(format!("Failed to load tax config for year {}: {}", year, e.to_string())))?;

    Ok(TaxRatesForValidation {
        cpp_employee_rate: tax_config.cpp.employee_rate.to_string().parse().unwrap_or(0.0),
        cpp_basic_exemption: tax_config.cpp.basic_exemption.to_string().parse().unwrap_or(0.0),
        cpp_ympe: tax_config.cpp.max_pensionable_earnings.to_string().parse().unwrap_or(0.0),
        cpp_max_contribution: tax_config.cpp.max_contribution.to_string().parse().unwrap_or(0.0),
        cpp2_rate: tax_config.cpp2.rate.to_string().parse().unwrap_or(0.0),
        cpp2_max_earnings: tax_config.cpp2.max_earnings.to_string().parse().unwrap_or(0.0),
        cpp2_max_contribution: tax_config.cpp2.max_contribution.to_string().parse().unwrap_or(0.0),
        ei_rate: tax_config.ei.employee_rate.to_string().parse().unwrap_or(0.0),
        ei_max_insurable_earnings: tax_config.ei.max_insurable_earnings.to_string().parse().unwrap_or(0.0),
        ei_max_contribution: tax_config.ei.max_contribution.to_string().parse().unwrap_or(0.0),
    })
}

/// CSV row structure for employee import/export
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct EmployeeCsvRow {
    employee_number: String,
    first_name: String,
    last_name: String,
    sin: String,
    address_street: String,
    address_city: String,
    address_province: String,
    address_postal_code: String,
    pay_type: String,
    pay_rate: String,
    date_of_birth: String,
    vacation_pay_rate: String,
    overtime_multiplier: String,
    t4127_code: String,
    additional_tax_amount: String,
    hire_date: String,
    hire_province: String,
    termination_date: String,
    is_active: String,
    notes: String,
    personal_amount_year: String,
    personal_amount_federal: String,
    personal_amount_provincial: String,
}

/// Export employees to CSV file
#[tauri::command]
pub async fn export_employees_csv(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<usize, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_all()?;
        let mut writer = csv::Writer::from_path(&file_path)
            .map_err(|e| CommandError::new(format!("Failed to create CSV file: {}", e)))?;
        
        for employee in &employees {
            // Get autofill values for this employee (for additional_tax_amount export)
            let autofills = db.employees().get_autofill(employee.id.unwrap_or(0))?;
            let additional_tax = autofills.iter()
                .find(|a| a.autofill_type == cpr_core::models::AutofillType::Deduction
                       && a.type_name.to_lowercase() == "additional_tax")
                .map(|a| a.amount.to_string())
                .unwrap_or_else(|| "0.00".to_string());
            
            // Get latest personal amount for this employee
            let (pa_year, pa_federal, pa_provincial) = match db.personal_amount()
                .get_latest_for_employee(employee.id.unwrap_or(0)) {
                Ok(Some(pa)) => (pa.year.to_string(), pa.federal_amount.to_string(), pa.provincial_amount.to_string()),
                _ => (String::new(), "0.00".to_string(), "0.00".to_string()),
            };
            
            let row = EmployeeCsvRow {
                employee_number: employee.employee_number.clone(),
                first_name: employee.first_name.clone(),
                last_name: employee.last_name.clone(),
                sin: employee.sin.clone(),
                address_street: employee.address.street.clone(),
                address_city: employee.address.city.clone(),
                address_province: employee.address.province.clone(),
                address_postal_code: employee.address.postal_code.clone(),
                pay_type: employee.pay_type.as_str().to_string(),
                pay_rate: employee.pay_rate.to_string(),
                date_of_birth: employee.date_of_birth.to_string(),
                vacation_pay_rate: employee.vacation_pay_rate.to_string(),
                overtime_multiplier: employee.overtime_multiplier.to_string(),
                t4127_code: "1".to_string(), // Default T4127 code
                additional_tax_amount: additional_tax,
                hire_date: employee.hire_date.to_string(),
                hire_province: employee.hire_province.clone(),
                termination_date: employee.termination_date.map(|d| d.to_string()).unwrap_or_default(),
                is_active: employee.is_active.to_string(),
                notes: employee.notes.clone().unwrap_or_default(),
                personal_amount_year: pa_year,
                personal_amount_federal: pa_federal,
                personal_amount_provincial: pa_provincial,
            };
            writer.serialize(row)
                .map_err(|e| CommandError::new(format!("Failed to write CSV row: {}", e)))?;
        }
        
        writer.flush()
            .map_err(|e| CommandError::new(format!("Failed to flush CSV writer: {}", e)))?;
        
        Ok(employees.len())
    })
}

/// Helper function to get the latest available tax year from config files
fn get_latest_tax_year() -> Result<i32, CommandError> {
    use std::fs;
    use std::collections::HashSet;
    
    let config_dir = resolve_config_dir()
        .ok_or_else(|| CommandError::new("Config directory not found"))?;
    
    let mut years = HashSet::new();
    let entries = fs::read_dir(&config_dir)
        .map_err(|e| CommandError::new(&format!("Failed to read config directory: {}", e)))?;
    
    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with("tax_rates_") && filename.ends_with(".json") {
                let without_prefix = filename.strip_prefix("tax_rates_").unwrap();
                let year_part = without_prefix.split('_').next()
                    .or_else(|| without_prefix.strip_suffix(".json"));
                
                if let Some(year_str) = year_part {
                    let year_str = year_str.trim_end_matches(".json");
                    if let Ok(year) = year_str.parse::<i32>() {
                        years.insert(year);
                    }
                }
            }
        }
    }
    
    years.into_iter().max()
        .ok_or_else(|| CommandError::new("No tax configuration files found"))
}

/// Helper function to create a default personal amount for a new employee
fn create_default_personal_amount(
    db: &cpr_db::repository::Database,
    employee_id: i64,
    company_province: &Province,
) -> Result<(), CommandError> {
    // Get the latest tax year
    let latest_year = match get_latest_tax_year() {
        Ok(year) => year,
        Err(_) => {
            // If we can't determine the year, just skip creating personal amount
            return Ok(());
        }
    };
    
    // Load tax config for the latest year
    let tax_config = match load_tax_config(latest_year) {
        Ok(config) => config,
        Err(_) => return Ok(()), // Skip if can't load config
    };
    
    // Get basic personal amounts from config
    let federal_amount = tax_config.federal.basic_personal_amount;
    let provincial_amount = tax_config.provincial.province_configs
        .get(company_province)
        .map(|pc| pc.basic_personal_amount)
        .unwrap_or(rust_decimal::Decimal::ZERO);
    
    // Create personal amount record
    let mut personal_amount = PersonalAmount {
        id: None,
        employee_id,
        province: company_province.code().to_string(),
        year: latest_year,
        federal_amount,
        provincial_amount,
        indexed_at: Utc::now(),
    };
    
    // Insert into database
    let _ = db.personal_amount().create(&mut personal_amount);
    
    Ok(())
}

/// Import employees from CSV file
#[tauri::command]
pub async fn import_employees_csv(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ImportResult, CommandError> {
    with_db(&state, |db| {
        // Get company information to determine province for personal amounts
        let company = db.company().get()
            .map_err(|e| CommandError::new(&format!("Failed to get company: {}", e)))?
            .ok_or_else(|| CommandError::new("No company configured"))?;
        let company_province = company.province;
        
        let mut reader = csv::Reader::from_path(&file_path)
            .map_err(|e| CommandError::new(format!("Failed to open CSV file: {}", e)))?;
        
        let mut created_count = 0;
        let mut updated_count = 0;
        let skipped_count = 0;
        let mut errors: Vec<EmployeeImportError> = Vec::new();
        
        for (line_num, result) in reader.records().enumerate() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    errors.push(EmployeeImportError {
                        employee_number: "".to_string(),
                        employee_name: "".to_string(),
                        error: format!("Line {}: Failed to read record: {}", line_num + 2, e),
                    });
                    continue;
                }
            };
            
            // Parse CSV row
            let row: EmployeeCsvRow = match record.deserialize(None) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(EmployeeImportError {
                        employee_number: "".to_string(),
                        employee_name: "".to_string(),
                        error: format!("Line {}: Failed to parse record: {}", line_num + 2, e),
                    });
                    continue;
                }
            };
            
            // Check if employee with this number already exists
            let existing_employee = db.employees().find_by_number(&row.employee_number)?;
            
            // Parse fields
            let pay_type = match PayType::from_str(&row.pay_type) {
                Some(pt) => pt,
                None => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid pay type '{}'", line_num + 2, row.pay_type),
                    });
                    continue;
                }
            };
            
            let pay_rate: rust_decimal::Decimal = match row.pay_rate.parse() {
                Ok(r) => r,
                Err(_) => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid pay rate '{}'", line_num + 2, row.pay_rate),
                    });
                    continue;
                }
            };
            
            let date_of_birth = match chrono::NaiveDate::parse_from_str(&row.date_of_birth, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid date of birth '{}'", line_num + 2, row.date_of_birth),
                    });
                    continue;
                }
            };
            
            let vacation_pay_rate: rust_decimal::Decimal = match row.vacation_pay_rate.parse() {
                Ok(r) => r,
                Err(_) => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid vacation pay rate '{}'", line_num + 2, row.vacation_pay_rate),
                    });
                    continue;
                }
            };
            
            let overtime_multiplier: rust_decimal::Decimal = match row.overtime_multiplier.parse() {
                Ok(r) => r,
                Err(_) => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid overtime multiplier '{}'", line_num + 2, row.overtime_multiplier),
                    });
                    continue;
                }
            };
            
            let hire_date = match chrono::NaiveDate::parse_from_str(&row.hire_date, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    errors.push(EmployeeImportError {
                        employee_number: row.employee_number.clone(),
                        employee_name: format!("{} {}", row.first_name, row.last_name),
                        error: format!("Line {}: Invalid hire date '{}'", line_num + 2, row.hire_date),
                    });
                    continue;
                }
            };
            
            let termination_date = if row.termination_date.is_empty() {
                None
            } else {
                match chrono::NaiveDate::parse_from_str(&row.termination_date, "%Y-%m-%d") {
                    Ok(d) => Some(d),
                    Err(_) => {
                        errors.push(EmployeeImportError {
                            employee_number: row.employee_number.clone(),
                            employee_name: format!("{} {}", row.first_name, row.last_name),
                            error: format!("Line {}: Invalid termination date '{}'", line_num + 2, row.termination_date),
                        });
                        continue;
                    }
                }
            };
            
            let is_active = row.is_active.to_lowercase() == "true" || row.is_active == "1";
            
            let notes = if row.notes.is_empty() {
                None
            } else {
                Some(row.notes.clone())
            };
            
            // Create employee
            let mut employee = Employee {
                id: None,
                employee_number: row.employee_number.clone(),
                first_name: row.first_name.clone(),
                last_name: row.last_name.clone(),
                sin: row.sin.clone(),
                address: cpr_core::models::Address {
                    street: row.address_street.clone(),
                    city: row.address_city.clone(),
                    province: row.address_province.clone(),
                    postal_code: row.address_postal_code.clone(),
                },
                pay_type,
                pay_rate,
                date_of_birth,
                vacation_pay_rate,
                vacation_balance: Decimal::ZERO,  // New employees start with zero balance
                vacation_balance_days: Decimal::ZERO,  // New employees start with zero days balance
                overtime_multiplier,
                ei_exempt: false,
                cpp_exempt: false,
                hire_date,
                hire_province: row.hire_province.clone(),
                termination_date,
                dental_benefit: 1,  // Default: no dental benefit
                is_active,
                created_at: chrono::Utc::now(),
                notes,
            };
            
            // Validate employee
            if let Err(e) = employee.validate() {
                errors.push(EmployeeImportError {
                    employee_number: row.employee_number.clone(),
                    employee_name: format!("{} {}", row.first_name, row.last_name),
                    error: format!("Line {}: Validation failed: {}", line_num + 2, e),
                });
                continue;
            }
            
            // Parse additional_tax_amount for autofill
            let additional_tax_amount: rust_decimal::Decimal = row.additional_tax_amount.parse().unwrap_or(rust_decimal::Decimal::ZERO);
            
            // Parse personal amount values (may be empty for older CSVs)
            let pa_year: i32 = row.personal_amount_year.parse().unwrap_or(0);
            let pa_federal: rust_decimal::Decimal = row.personal_amount_federal.parse().unwrap_or(rust_decimal::Decimal::ZERO);
            let pa_provincial: rust_decimal::Decimal = row.personal_amount_provincial.parse().unwrap_or(rust_decimal::Decimal::ZERO);
            let has_personal_amounts = !row.personal_amount_federal.is_empty() || !row.personal_amount_provincial.is_empty();
            
            // Determine if we're creating new or updating existing
            if let Some(existing) = existing_employee {
                // Update existing employee: preserve ID and created_at, update other fields
                employee.id = existing.id;
                employee.created_at = existing.created_at;
                
                // Update pay rate history if changed (similar to update_employee command)
                if existing.pay_rate != employee.pay_rate || existing.pay_type != employee.pay_type {
                    let today = chrono::Utc::now().date_naive();
                    if let Err(e) = db.history().close_pay_rate_history(employee.id.unwrap(), today) {
                        // Log error but continue
                        errors.push(EmployeeImportError {
                            employee_number: row.employee_number.clone(),
                            employee_name: format!("{} {}", row.first_name, row.last_name),
                            error: format!("Line {}: Warning - Failed to close pay rate history: {}", line_num + 2, e),
                        });
                    } else {
                        let mut new_history = cpr_core::models::PayRateHistory::new(
                            employee.id.unwrap(),
                            employee.pay_rate.clone(),
                            employee.pay_type,
                            today,
                            Some("Pay rate updated via CSV import".to_string()),
                        );
                        if let Err(e) = db.history().add_pay_rate(&mut new_history) {
                            errors.push(EmployeeImportError {
                                employee_number: row.employee_number.clone(),
                                employee_name: format!("{} {}", row.first_name, row.last_name),
                                error: format!("Line {}: Warning - Failed to add pay rate history: {}", line_num + 2, e),
                            });
                        }
                    }
                }
                
                // Update employment history if hire/termination changed
                if existing.hire_date != employee.hire_date || existing.termination_date != employee.termination_date {
                    if let Ok(Some(mut curr_eh)) = db.history().get_current_employment(employee.id.unwrap()) {
                        curr_eh.hire_date = employee.hire_date;
                        if let Some(term_date) = employee.termination_date {
                            curr_eh.terminate(term_date, Some("Employment updated via CSV import".to_string()), employee.is_active);
                        } else {
                            curr_eh.termination_date = None;
                            curr_eh.termination_reason = None;
                            curr_eh.rehire_eligible = true;
                        }
                        if let Err(e) = db.history().update_employment(&curr_eh) {
                            errors.push(EmployeeImportError {
                                employee_number: row.employee_number.clone(),
                                employee_name: format!("{} {}", row.first_name, row.last_name),
                                error: format!("Line {}: Warning - Failed to update employment history: {}", line_num + 2, e),
                            });
                        }
                    } else {
                        let mut new_eh = cpr_core::models::EmploymentHistory::new(employee.id.unwrap(), employee.hire_date);
                        if let Some(term_date) = employee.termination_date {
                            new_eh.terminate(term_date, Some("Employment updated via CSV import".to_string()), employee.is_active);
                        }
                        if let Err(e) = db.history().add_employment(&mut new_eh) {
                            errors.push(EmployeeImportError {
                                employee_number: row.employee_number.clone(),
                                employee_name: format!("{} {}", row.first_name, row.last_name),
                                error: format!("Line {}: Warning - Failed to create employment history: {}", line_num + 2, e),
                            });
                        }
                    }
                }
                
                // Update employee record
                match db.employees().update(&employee) {
                    Ok(_) => {
                        updated_count += 1;
                        
                        // Update or create autofill for additional_tax_amount
                        if additional_tax_amount > rust_decimal::Decimal::ZERO {
                            // Check if autofill exists and update, or create new
                            let existing_autofills = db.employees().get_autofill(employee.id.unwrap())?;
                            let existing_deduction = existing_autofills.iter()
                                .find(|a| a.autofill_type == cpr_core::models::AutofillType::Deduction
                                    && a.type_name.to_lowercase() == "additional_tax");
                            
                            let mut autofill = if let Some(existing) = existing_deduction {
                                let mut a = existing.clone();
                                a.amount = additional_tax_amount;
                                a
                            } else {
                                cpr_core::models::EmployeeAutofill::new(
                                    employee.id.unwrap(),
                                    cpr_core::models::AutofillType::Deduction,
                                    "additional_tax".to_string(),
                                    additional_tax_amount,
                                )
                            };
                            
                            if let Err(e) = db.employees().save_autofill(&mut autofill) {
                                errors.push(EmployeeImportError {
                                    employee_number: row.employee_number.clone(),
                                    employee_name: format!("{} {}", row.first_name, row.last_name),
                                    error: format!("Line {}: Warning - Failed to save autofill: {}", line_num + 2, e),
                                });
                            }
                        }
                        
                        // Update personal amount if provided in CSV
                        if has_personal_amounts {
                            let emp_id = employee.id.unwrap();
                            // Get the latest personal amount record for this employee
                            match db.personal_amount().get_latest_for_employee(emp_id) {
                                Ok(Some(mut existing_pa)) => {
                                    // Update the existing record with imported values
                                    if !row.personal_amount_federal.is_empty() {
                                        existing_pa.federal_amount = pa_federal;
                                    }
                                    if !row.personal_amount_provincial.is_empty() {
                                        existing_pa.provincial_amount = pa_provincial;
                                    }
                                    existing_pa.indexed_at = chrono::Utc::now();
                                    if let Err(e) = db.personal_amount().update(existing_pa.id.unwrap(), &existing_pa) {
                                        errors.push(EmployeeImportError {
                                            employee_number: row.employee_number.clone(),
                                            employee_name: format!("{} {}", row.first_name, row.last_name),
                                            error: format!("Line {}: Warning - Failed to update personal amount: {}", line_num + 2, e),
                                        });
                                    }
                                }
                                _ => {
                                    // No existing personal amount found - get the tax year and province to create one
                                    let year = if pa_year > 0 { pa_year } else { get_latest_tax_year().unwrap_or(chrono::Utc::now().year()) };
                                    let province = if !row.hire_province.is_empty() { &row.hire_province } else { company_province.code() };
                                    let mut new_pa = PersonalAmount {
                                        id: None,
                                        employee_id: emp_id,
                                        province: province.to_string(),
                                        year,
                                        federal_amount: pa_federal,
                                        provincial_amount: pa_provincial,
                                        indexed_at: chrono::Utc::now(),
                                    };
                                    if let Err(e) = db.personal_amount().create(&mut new_pa) {
                                        errors.push(EmployeeImportError {
                                            employee_number: row.employee_number.clone(),
                                            employee_name: format!("{} {}", row.first_name, row.last_name),
                                            error: format!("Line {}: Warning - Failed to create personal amount: {}", line_num + 2, e),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(EmployeeImportError {
                            employee_number: row.employee_number.clone(),
                            employee_name: format!("{} {}", row.first_name, row.last_name),
                            error: format!("Line {}: Failed to update employee: {}", line_num + 2, e),
                        });
                    }
                }
            } else {
                // Create new employee
                match db.employees().create(&mut employee) {
                    Ok(employee_id) => {
                        created_count += 1;
                        
                        // Create initial pay rate history
                        let today = chrono::Utc::now().date_naive();
                        let mut pay_history = cpr_core::models::PayRateHistory::new(
                            employee_id,
                            employee.pay_rate.clone(),
                            employee.pay_type,
                            today,
                            Some("Initial pay rate on import".to_string()),
                        );
                        let _ = db.history().add_pay_rate(&mut pay_history);
                        
                        // Create employment history
                        let mut emp_history = cpr_core::models::EmploymentHistory::new(
                            employee_id,
                            employee.hire_date,
                        );
                        let _ = db.history().add_employment(&mut emp_history);
                        
                        // Create default personal amount with company province
                        let _ = create_default_personal_amount(db, employee_id, &company_province);
                        
                        // Override personal amount with imported values if provided
                        if has_personal_amounts {
                            match db.personal_amount().get_latest_for_employee(employee_id) {
                                Ok(Some(mut pa)) => {
                                    if !row.personal_amount_federal.is_empty() {
                                        pa.federal_amount = pa_federal;
                                    }
                                    if !row.personal_amount_provincial.is_empty() {
                                        pa.provincial_amount = pa_provincial;
                                    }
                                    pa.indexed_at = chrono::Utc::now();
                                    if let Err(e) = db.personal_amount().update(pa.id.unwrap(), &pa) {
                                        errors.push(EmployeeImportError {
                                            employee_number: row.employee_number.clone(),
                                            employee_name: format!("{} {}", row.first_name, row.last_name),
                                            error: format!("Line {}: Warning - Failed to update personal amount: {}", line_num + 2, e),
                                        });
                                    }
                                }
                                _ => {
                                    // No personal amount found - create one with imported values
                                    let year = if pa_year > 0 { pa_year } else { get_latest_tax_year().unwrap_or(chrono::Utc::now().year()) };
                                    let province = if !row.hire_province.is_empty() { &row.hire_province } else { company_province.code() };
                                    let mut new_pa = PersonalAmount {
                                        id: None,
                                        employee_id,
                                        province: province.to_string(),
                                        year,
                                        federal_amount: pa_federal,
                                        provincial_amount: pa_provincial,
                                        indexed_at: chrono::Utc::now(),
                                    };
                                    if let Err(e) = db.personal_amount().create(&mut new_pa) {
                                        errors.push(EmployeeImportError {
                                            employee_number: row.employee_number.clone(),
                                            employee_name: format!("{} {}", row.first_name, row.last_name),
                                            error: format!("Line {}: Warning - Failed to create personal amount: {}", line_num + 2, e),
                                        });
                                    }
                                }
                            }
                        }
                        
                        // Create autofill entry for additional_tax_amount if > 0
                        if additional_tax_amount > rust_decimal::Decimal::ZERO {
                            let mut autofill = cpr_core::models::EmployeeAutofill::new(
                                employee_id,
                                cpr_core::models::AutofillType::Deduction,
                                "additional_tax".to_string(),
                                additional_tax_amount,
                            );
                            if let Err(e) = db.employees().save_autofill(&mut autofill) {
                                // Log the error but don't fail the import
                                errors.push(EmployeeImportError {
                                    employee_number: row.employee_number.clone(),
                                    employee_name: format!("{} {}", row.first_name, row.last_name),
                                    error: format!("Line {}: Warning - Failed to create autofill for additional_tax_amount: {}", line_num + 2, e),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(EmployeeImportError {
                            employee_number: row.employee_number.clone(),
                            employee_name: format!("{} {}", row.first_name, row.last_name),
                            error: format!("Line {}: Failed to create employee: {}", line_num + 2, e),
                        });
                    }
                }
            }
        }
        
        Ok(ImportResult {
            created: created_count,
            updated: updated_count,
            skipped: skipped_count,
            errors,
        })
    })
}

/// Result structure for CSV import
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<EmployeeImportError>,
}

/// Structured error information for employee import
#[derive(Debug, Serialize)]
pub struct EmployeeImportError {
    pub employee_number: String,
    pub employee_name: String,
    pub error: String,
}

// =============================================
// Employee Autofill Commands
// =============================================

/// Get all autofill values for an employee
#[tauri::command]
pub async fn get_employee_autofill(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<cpr_core::models::EmployeeAutofill>, CommandError> {
    with_db(&state, |db| {
        let autofills = db.employees().get_autofill(employee_id)?;
        Ok(autofills)
    })
}

/// Get active autofill values for an employee
#[tauri::command]
pub async fn get_active_employee_autofill(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<cpr_core::models::EmployeeAutofill>, CommandError> {
    with_db(&state, |db| {
        let autofills = db.employees().get_active_autofill(employee_id)?;
        Ok(autofills)
    })
}

/// Save (create or update) an autofill entry
#[tauri::command]
pub async fn save_employee_autofill(
    mut autofill: cpr_core::models::EmployeeAutofill,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let id = db.employees().save_autofill(&mut autofill)?;
        Ok(id)
    })
}

/// Delete an autofill entry
#[tauri::command]
pub async fn delete_employee_autofill(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        db.employees().delete_autofill(id)?;
        Ok(())
    })
}

/// Delete all autofill entries for an employee
#[tauri::command]
pub async fn delete_all_employee_autofill(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        db.employees().delete_all_autofill(employee_id)?;
        Ok(())
    })
}
