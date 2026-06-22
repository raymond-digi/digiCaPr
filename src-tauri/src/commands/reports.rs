use tauri::State;
use crate::state::AppState;
use crate::error::CommandError;
use std::path::PathBuf;
use std::fs;
use rust_decimal::Decimal;
use chrono::{NaiveDate, Datelike};


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

/// Build the output directory by replacing the first path component
/// of `output_dir` with the database file stem.
/// e.g. output_dir="reports/paystubs" with db="company1.db" → "{db_parent}/company1/paystubs"
fn get_output_dir(state: &State<AppState>, output_dir: &str) -> Result<PathBuf, CommandError> {
    let db_path = state.get_db_path()
        .ok_or_else(|| CommandError::new("No database open"))?;

    let db = PathBuf::from(&db_path);
    let db_parent = db.parent()
        .ok_or_else(|| CommandError::new("Invalid database path"))?;
    let db_stem = db.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("reports");

    // Strip the first path component from output_dir and replace with db_stem
    let components: Vec<&str> = output_dir
        .split(|c| c == '/' || c == '\\')
        .filter(|s| !s.is_empty())
        .collect();

    let mut result = db_parent.join(db_stem);
    // Append remaining path components (skip the first one which we replaced)
    for component in components.into_iter().skip(1) {
        result = result.join(component);
    }

    Ok(result)
}

/// Generate current payroll report PDF
#[tauri::command]
pub async fn generate_payroll_report(
    payroll_ids: Vec<i64>,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    let reports_dir = get_output_dir(&state, &output_dir)?;
    
    with_db(&state, |db| {
        fs::create_dir_all(&reports_dir)
            .map_err(|e| CommandError::new(format!("Failed to create output directory: {}", e)))?;

        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;

        let mut payrolls = Vec::new();
        let mut employees = Vec::new();

        let mut period_start = NaiveDate::from_ymd_opt(9999, 12, 31).unwrap();
        let mut period_end = NaiveDate::from_ymd_opt(0001, 1, 1).unwrap();
        let mut pay_date = NaiveDate::from_ymd_opt(0001, 1, 1).unwrap();

        for payroll_id in payroll_ids {
            let payroll = db.payroll().get(payroll_id)?;
            payrolls.push(payroll.clone());

            let emp_id = payroll.employee_id;
            if let Ok(employee) = db.employees().get(emp_id) {
                employees.push(employee);
            }

            if payroll.pay_period_start < period_start {
                period_start = payroll.pay_period_start;
            }
            if payroll.pay_period_end > period_end {
                period_end = payroll.pay_period_end;
            }
            if payroll.pay_date > pay_date {
                pay_date = payroll.pay_date;
            }
        }

        let filename = format!("payroll_{}_{}_{}.pdf",
            pay_date.format("%Y%m%d"),
            period_start.format("%m%d"),
            period_end.format("%m%d")
        );
        let path = reports_dir.join(filename);

        cpr_reports::payroll_report::generate_payroll_report(
            &path,
            &payrolls,
            &employees,
            &company,
            period_start,
            period_end,
            pay_date,
        ).map_err(|e| CommandError::new(format!("Failed to generate summary report: {}", e)))?;

        Ok(path.to_string_lossy().to_string())
    })
}

/// Generate history payroll report PDF
#[tauri::command]
pub async fn generate_history_payroll_report(
    payroll_ids: Vec<i64>,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    let reports_dir = get_output_dir(&state, &output_dir)?;
    
    with_db(&state, |db| {
        fs::create_dir_all(&reports_dir)
            .map_err(|e| CommandError::new(format!("Failed to create output directory: {}", e)))?;

        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;

        let mut payrolls = Vec::new();
        let mut employees = Vec::new();

        let mut period_start = NaiveDate::from_ymd_opt(9999, 12, 31).unwrap();
        let mut period_end = NaiveDate::from_ymd_opt(0001, 1, 1).unwrap();
        let mut pay_date = NaiveDate::from_ymd_opt(0001, 1, 1).unwrap();

        for payroll_id in payroll_ids {
            let payroll = db.payroll_history().get(payroll_id)?;
            payrolls.push(payroll.clone());

            let emp_id = payroll.employee_id;
            if let Ok(employee) = db.employees().get(emp_id) {
                employees.push(employee);
            }

            if payroll.pay_period_start < period_start {
                period_start = payroll.pay_period_start;
            }
            if payroll.pay_period_end > period_end {
                period_end = payroll.pay_period_end;
            }
            if payroll.pay_date > pay_date {
                pay_date = payroll.pay_date;
            }
        }

        // Use the employee number from the first employee to make filename unique per employee
        let emp_number = employees.first()
            .map(|e| e.employee_number.as_str())
            .unwrap_or("unknown");
        let filename = format!("history_{}_{}_{}_{}.pdf",
            emp_number,
            pay_date.format("%Y%m%d"),
            period_start.format("%m%d"),
            period_end.format("%m%d")
        );
        let path = reports_dir.join(filename);

        cpr_reports::payroll_report::generate_payroll_report(
            &path,
            &payrolls,
            &employees,
            &company,
            period_start,
            period_end,
            pay_date,
        ).map_err(|e| CommandError::new(format!("Failed to generate history summary report: {}", e)))?;

        Ok(path.to_string_lossy().to_string())
    })
}

#[tauri::command]
pub async fn generate_remittance_report(
    remittance_id: i64,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    let reports_dir = get_output_dir(&state, &output_dir)?;
    
    with_db(&state, |db| {
        fs::create_dir_all(&reports_dir)
            .map_err(|e| CommandError::new(format!("Failed to create output directory: {}", e)))?;

        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;

        let remittance = db.remittance().get(remittance_id)?;

        let payrolls = db.payroll_history().list_by_remittance(remittance_id)?;
        let mut employees = Vec::new();
        for payroll in &payrolls {
            if let Ok(employee) = db.employees().get(payroll.employee_id) {
                employees.push(employee);
            }
        }

        let filename = format!(
            "remittance_{}_{}.pdf",
            remittance.period_start.format("%Y%m%d"),
            remittance.period_end.format("%Y%m%d")
        );
        let path = reports_dir.join(filename);

        cpr_reports::remittance_report::generate_remittance_report(
            &path,
            &payrolls,
            &employees,
            &company,
            remittance.period_start,
            remittance.period_end,
        ).map_err(|e| CommandError::new(format!("Failed to generate remittance report: {}", e)))?;

        Ok(path.to_string_lossy().to_string())
    })
}

/// Generate paystubs for multiple payrolls
#[tauri::command]
pub async fn generate_payroll_paystubs(
    payroll_ids: Vec<i64>,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, CommandError> {
    let reports_dir = get_output_dir(&state, &output_dir)?;
    
    with_db(&state, |db| {
        // Ensure output directory exists
        fs::create_dir_all(&reports_dir)
            .map_err(|e| CommandError::new(format!("Failed to create output directory: {}", e)))?;

        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;

        let mut generated_files = Vec::new();

        for payroll_id in payroll_ids {
            let payroll = db.payroll().get(payroll_id)?;
            let employee = db.employees().get(payroll.employee_id)?;

            // Fetch YTD totals for the employee for the pay date year
            let year = payroll.pay_date.year() as i32;
            let ytd = db.ytd().get_or_create(payroll.employee_id, year)?;

            let filename = format!(
                "paystub_{}_{}.pdf",
                employee.employee_number,
                payroll.pay_date.format("%Y%m%d")
            );
            let path = reports_dir.join(filename);

            cpr_reports::payroll_paystub::generate_paystub(&path, &employee, &payroll, &ytd, &company)
                .map_err(|e| CommandError::new(format!(
                    "Failed to generate paystub for payroll {} (employee {}): {}",
                    payroll_id, employee.employee_number, e
                )))?;

            generated_files.push(path.to_string_lossy().to_string());
        }

        Ok(generated_files)
    })
}

/// Generate a paystub PDF and return the file path
/// Note: The current implementation needs a file path, so we'll generate to a temp location
#[tauri::command]
pub async fn generate_payroll_paystub(
    payroll_id: i64,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        // Get payroll and employee data
        let payroll = db.payroll().get(payroll_id)?;
        let employee = db.employees().get(payroll.employee_id)?;
        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;
        
        // Fetch YTD totals for the employee for the pay date year
        let year = payroll.pay_date.year() as i32;
        let ytd = db.ytd().get_or_create(payroll.employee_id, year)?;
        
        // Generate PDF to file
        let path = PathBuf::from(&output_path);
        cpr_reports::payroll_paystub::generate_paystub(&path, &employee, &payroll, &ytd, &company)
            .map_err(|e| CommandError::new(format!("Failed to generate paystub: {}", e)))?;
        
        Ok(output_path)
    })
}

/// Generate T4 form for an employee for a specific year
/// Returns file path where PDF was saved
#[tauri::command]
pub async fn generate_t4(
    employee_id: i64,
    year: i32,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        // Get employee and company data
        let employee = db.employees().get(employee_id)?;
        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;
        
        let t4_repo = db.t4();
        let slip = t4_repo.get_or_create_slip(employee_id, year)?;
        let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
        let t4_data = build_t4_data_from_slip(&employee, year, &box_values, slip.net_pay)
            .ok_or_else(|| CommandError::new(format!("No T4 data for employee {} year {}", employee_id, year)))?;
        
        // Generate T4 to file
        let path = PathBuf::from(&output_path);
        cpr_reports::t4::generate_t4(&path, &t4_data, &company)
            .map_err(|e| CommandError::new(format!("Failed to generate T4: {}", e)))?;
        
        Ok(output_path)
    })
}

/// Export payroll data to CSV and save to file
#[tauri::command]
pub async fn export_payroll_csv(
    year: i32,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        // Get all payroll records for the year
        let start_date = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap();
        let payroll_records = db.payroll().list_by_date_range(start_date, end_date)?;
        
        // Generate CSV manually (since there's no csv module in cpr-reports yet)
        let mut csv = String::from("Employee ID,Pay Period Start,Pay Period End,Pay Date,Hours,Gross Pay,CPP,CPP2,EI,Federal Tax,Provincial Tax,Net Pay\n");
        
        for p in payroll_records {
            let total_hours = p.regular_hours.unwrap_or(Decimal::ZERO) + p.overtime_hours.unwrap_or(Decimal::ZERO);
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                p.employee_id,
                p.pay_period_start,
                p.pay_period_end,
                p.pay_date,
                total_hours.to_string(),
                p.gross_pay,
                p.deductions.cpp,
                p.deductions.cpp2,
                p.deductions.ei,
                p.deductions.federal_tax,
                p.deductions.provincial_tax,
                p.net_pay,
            ));
        }
        
        // Write to file
        std::fs::write(&output_path, csv)
            .map_err(|e| CommandError::new(format!("Failed to write CSV: {}", e)))?;
        
        Ok(output_path)
    })
}


/// Generate T4s for all employees for a specific year
#[tauri::command]
pub async fn generate_payroll_t4(
    year: i32,
    output_dir: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, CommandError> {
    let t4_dir = get_output_dir(&state, &output_dir)?;
    fs::create_dir_all(&t4_dir)
        .map_err(|e| CommandError::new(format!("Failed to create output directory: {}", e)))?;

    with_db(&state, |db| {
        let employees = db.employees().list_all()?;
        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;
        
        let mut generated_files = Vec::new();
        
        for employee in employees.iter().filter(|e| e.is_active) {
            // Get latest T4 slip with box values
            let t4_repo = db.t4();
            let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)?;
            let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
            
            // Build T4 data from box values
            let t4_data = match build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
                Some(data) => data,
                None => continue,
            };
             
            // Generate file path
            let filename = format!("T4_{}_{}_{}.pdf", year, employee.employee_number, employee.last_name);
            let path = t4_dir.join(filename);
            
            // Generate T4
            cpr_reports::t4::generate_t4(&path, &t4_data, &company)
                .map_err(|e| CommandError::new(format!("Failed to generate T4 for {}: {}", employee.employee_number, e)))?;
            
            generated_files.push(path.to_string_lossy().to_string());
        }
        
        Ok(generated_files)
    })
}

/// Generate personal amount completeness report (employees missing personal amounts for a year)
#[tauri::command]
pub async fn generate_personal_amount_report(
    year: i32,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_active()?;
        let mut missing = Vec::new();

        for employee in employees {
            let tax_province_code = employee.hire_province.trim().to_string();
            if db.personal_amount().get_by_employee_province_year(employee.id.unwrap(), &tax_province_code, year).map_err(CommandError::from)?.is_none() {
                missing.push(format!(
                    "{} - {} {} (ID: {}) - Missing personal amount for {}",
                    employee.employee_number,
                    employee.first_name,
                    employee.last_name,
                    employee.id.unwrap(),
                    tax_province_code
                ));
            }
        }

        let csv = format!("Employees Missing Personal Amounts for {}:\n\n", year);
        let csv_content = missing.join("\n");

        std::fs::write(&output_path, format!("{}\n{}", csv, csv_content))
            .map_err(|e| CommandError::new(format!("Failed to write report: {}", e)))?;
Ok(output_path)
})
}

/// Helper: Build T4Data from flexible schema box values
/// `net_pay_from_payroll` is the sum of net_pay from payroll history (ground truth)
fn build_t4_data_from_slip(
    employee: &cpr_core::models::Employee,
    year: i32,
    box_values: &[cpr_db::repository::t4::T4BoxValue],
    net_pay_from_payroll: rust_decimal::Decimal,
) -> Option<cpr_reports::t4::T4Data> {
    if box_values.is_empty() {
        return None;
    }

    let mut employment_income = rust_decimal::Decimal::ZERO;
    let mut cpp_contributions = rust_decimal::Decimal::ZERO;
    let mut cpp2_contributions = rust_decimal::Decimal::ZERO;
    let mut rpp_contributions = rust_decimal::Decimal::ZERO;
    let mut ei_premiums = rust_decimal::Decimal::ZERO;
    let mut income_tax_deducted = rust_decimal::Decimal::ZERO;
    let mut ei_insurable_earnings = rust_decimal::Decimal::ZERO;
    let mut cpp_pensionable_earnings = rust_decimal::Decimal::ZERO;
    let mut pension_adjustment = rust_decimal::Decimal::ZERO;
    let mut dental_benefit = employee.dental_benefit;

    for bv in box_values {
        let value = bv.final_value();
        match bv.box_type.as_str() {
            "box_14" => employment_income = value,
            "box_16" => cpp_contributions = value,
            "box_16a" => cpp2_contributions = value,
            "box_20" => rpp_contributions = value,
            "box_18" => ei_premiums = value,
            "box_22" => income_tax_deducted = value,
            "box_24" => ei_insurable_earnings = value,
            "box_26" => cpp_pensionable_earnings = value,
            "box_45" => dental_benefit = value.to_string().parse().unwrap_or(1),
            "box_52" => pension_adjustment = value,
            _ => {}
        }
    }

    // Skip if no payroll data (all zeros)
    if employment_income.is_zero() {
        return None;
    }

    // Computed net pay from box values for comparison
    let computed_net_pay = employment_income - cpp_contributions - cpp2_contributions
        - ei_premiums - income_tax_deducted - rpp_contributions;

    Some(cpr_reports::t4::T4Data {
        employee: employee.clone(),
        year,
        employment_income,
        cpp_contributions,
        cpp2_contributions,
        rpp_contributions,
        ei_premiums,
        income_tax_deducted,
        ei_insurable_earnings,
        cpp_pensionable_earnings,
        pension_adjustment,
        dental_benefit,
        employment_code: Some(employee.employee_number.clone()),
        province_of_employment: employee.hire_province.to_string(),
        net_pay: net_pay_from_payroll,
        computed_net_pay: computed_net_pay,
    })
}

/// Calculate T4 values for all employees for a specific year
/// Creates/updates t4_slip records and returns the T4 slip data
#[tauri::command]
pub async fn calculate_t4_for_year(
year: i32,
state: State<'_, AppState>,
) -> Result<Vec<cpr_reports::t4::T4Data>, CommandError> {
with_db(&state, |db| {
let employees = db.employees().list_all()?;
let _company = db.company().get()?
    .ok_or_else(|| CommandError::new("Company information not set"))?;

let t4_repo = db.t4();
// Calculate T4s from YTD for all employees
t4_repo.calculate_and_save_for_year(year, "system")
    .map_err(|e| CommandError::new(format!("Failed to calculate T4s: {}", e)))?;

let mut t4_slips = Vec::new();

for employee in employees.iter().filter(|e| e.is_active) {
    let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)
        .map_err(|e| CommandError::new(format!("Failed to get T4 slip: {}", e)))?;
    
    let box_values = t4_repo.get_box_values(slip.id.unwrap())
        .map_err(|e| CommandError::new(format!("Failed to get box values: {}", e)))?;
    
    if let Some(t4_data) = build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
        t4_slips.push(t4_data);
    }
}

Ok(t4_slips)
})
}

/// Generate T4 summary PDF for all employees in a year
#[tauri::command]
pub async fn generate_t4_summary_pdf(
year: i32,
output_path: String,
state: State<'_, AppState>,
) -> Result<String, CommandError> {
with_db(&state, |db| {
let company = db.company().get()?
    .ok_or_else(|| CommandError::new("Company information not set"))?;

let employees = db.employees().list_all()?;
let t4_repo = db.t4();

let mut t4_slips = Vec::new();
for employee in employees.iter().filter(|e| e.is_active) {
    let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)?;
    let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
    if let Some(t4_data) = build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
        t4_slips.push(t4_data);
    }
}

// Get total remittances paid for the year (Box 82)
let remittances = db.remittance().list_by_year(year)
    .unwrap_or_default();
let total_remittances: rust_decimal::Decimal = remittances.iter()
    .map(|r| r.grand_total)
    .sum();

let path = PathBuf::from(&output_path);
cpr_reports::t4::generate_t4_summary_with_remittances(&path, year, &company, &t4_slips, total_remittances)
    .map_err(|e| CommandError::new(format!("Failed to generate T4 summary: {}", e)))?;

Ok(output_path)
})
}

/// Export T4 XML efile for a year in CRA T619 Internet File Transfer format
/// Transmitter info is fetched from registry with key paths like "transmitter/bn15", "transmitter/name", etc.
#[tauri::command]
pub async fn export_t4_xml(
    year: i32,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
        let company = db.company().get()?
            .ok_or_else(|| CommandError::new("Company information not set"))?;

        let employees = db.employees().list_all()?;
        let t4_repo = db.t4();

        let mut t4_slips = Vec::new();
        for employee in employees.iter().filter(|e| e.is_active) {
            let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)?;
            let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
            if let Some(t4_data) = build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
                t4_slips.push(t4_data);
            }
        }

        // Fetch transmitter info from registry
        let registry = db.registry();
        let transmitter = cpr_reports::t4::T619Transmitter {
            bn15: registry.get("transmitter/bn15")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_else(|| company.business_number.clone().unwrap_or_default()),
            name: registry.get("transmitter/name")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_else(|| company.name.clone()),
            contact_name: registry.get("transmitter/contact_name")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_default(),
            phone_area: registry.get("transmitter/phone_area")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_default(),
            phone: registry.get("transmitter/phone")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_default(),
            email: registry.get("transmitter/email")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                }),
            submission_ref_id: registry.get("transmitter/submission_ref")
                .ok()
                .flatten()
                .and_then(|e| match e.value {
                    cpr_core::models::RegistryValue::String(s) => Some(s),
                    _ => None,
                }),
        };

        let path = PathBuf::from(&output_path);
        cpr_reports::t4::generate_t4_xml(&path, year, &company, &t4_slips, &transmitter)
            .map_err(|e| CommandError::new(format!("Failed to generate T4 XML: {}", e)))?;

        Ok(output_path)
    })
}

/// Export T4 CSV efile for a year
#[tauri::command]
pub async fn export_t4_csv(
    year: i32,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    with_db(&state, |db| {
    let company = db.company().get()?
        .ok_or_else(|| CommandError::new("Company information not set"))?;

    let employees = db.employees().list_all()?;
    let t4_repo = db.t4();

    let mut t4_slips = Vec::new();
    for employee in employees.iter().filter(|e| e.is_active) {
        let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)?;
        let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
        if let Some(t4_data) = build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
            t4_slips.push(t4_data);
        }
    }

    let path = PathBuf::from(&output_path);
    cpr_reports::t4::generate_t4_csv(&path, year, &company, &t4_slips)
        .map_err(|e| CommandError::new(format!("Failed to generate T4 CSV: {}", e)))?;

    Ok(output_path)
    })
}

/// Get T4 summary data for a year (all summary boxes for the T4 Summary)
#[tauri::command]
pub async fn get_t4_summary(
    year: i32,
    state: State<'_, AppState>,
) -> Result<cpr_reports::t4::T4SummaryData, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_all()?;
        let t4_repo = db.t4();

        let mut t4_slips = Vec::new();
        for employee in employees.iter().filter(|e| e.is_active) {
            let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)?;
            let box_values = t4_repo.get_box_values(slip.id.unwrap())?;
            if let Some(t4_data) = build_t4_data_from_slip(employee, year, &box_values, slip.net_pay) {
                t4_slips.push(t4_data);
            }
        }

        // Get total remittances paid for the year (Box 82)
        let remittances = db.remittance().list_by_year(year)
            .unwrap_or_default();
        let total_remittances: rust_decimal::Decimal = remittances.iter()
            .map(|r| r.grand_total)
            .sum();

        let summary = cpr_reports::t4::T4SummaryData::calculate(year, &t4_slips, total_remittances);
        Ok(summary)
    })
}

