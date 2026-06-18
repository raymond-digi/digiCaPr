use tauri::State;
use serde::{Serialize, Deserialize};
use crate::state::AppState;
use crate::error::CommandError;
use cpr_db::repository::t4::{T4SlipRecord, T4BoxValue};
use rust_decimal::Decimal;

/// T4 box value update (matches frontend T4BoxValueUpdate type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4BoxValueUpdate {
    pub employee_id: i64,
    pub year: i32,
    pub box_14_adjustment: Decimal,
    pub box_16_adjustment: Decimal,
    pub box_16a_adjustment: Decimal,
    pub box_18_adjustment: Decimal,
    pub box_20_adjustment: Decimal,
    pub box_22_adjustment: Decimal,
    pub box_24_adjustment: Decimal,
    pub box_26_adjustment: Decimal,
    pub box_45_adjustment: Decimal,
    pub box_52_adjustment: Decimal,
}

fn with_db<F, T>(state: &State<AppState>, f: F) -> Result<T, CommandError>
where
    F: FnOnce(&cpr_db::repository::Database) -> Result<T, CommandError>,
{
    let db_guard = state.database.lock().unwrap();
    let db = db_guard.as_ref()
        .ok_or_else(|| CommandError::new("No database open"))?;
    f(db)
}

/// List T4 slips for a year
#[tauri::command]
pub async fn list_t4_slips_for_year(
    year: i32,
    state: State<'_, AppState>,
) -> Result<Vec<T4SlipRecord>, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        let slips = t4_repo.list_slips_by_year(year)?;
        Ok(slips)
    })
}

/// Get or create T4 slip for employee/year
#[tauri::command]
pub async fn get_or_create_t4_slip(
    employee_id: i64,
    year: i32,
    state: State<'_, AppState>,
) -> Result<T4SlipRecord, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        let slip = t4_repo.get_or_create_slip(employee_id, year)?;
        Ok(slip)
    })
}

/// Create new version of T4 slip for recalculation/amendment
#[tauri::command]
pub async fn create_t4_slip_version(
    employee_id: i64,
    year: i32,
    state: State<'_, AppState>,
) -> Result<T4SlipRecord, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        let slip = t4_repo.create_slip_version(employee_id, year)?;
        Ok(slip)
    })
}

/// Get box values for a T4 slip
#[tauri::command]
pub async fn get_t4_box_values(
    slip_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<T4BoxValue>, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        let values = t4_repo.get_box_values(slip_id)?;
        Ok(values)
    })
}

/// Save box value for a T4 slip
#[tauri::command]
pub async fn save_t4_box_value(
    mut box_value: T4BoxValue,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        t4_repo.save_box_value(&mut box_value)?;
        Ok(box_value.id.unwrap_or(0))
    })
}

/// Calculate and save T4 values for a year (new schema)
/// NOTE: The frontend now calls `calculate_t4_for_year` from reports.rs instead.
/// This command is kept for backward compatibility but should be removed in future.
#[tauri::command]
pub async fn calculate_t4_for_year_v2(
    year: i32,
    state: State<'_, AppState>,
) -> Result<Vec<T4SlipRecord>, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        t4_repo.calculate_and_save_for_year(year, "system")?;
        let slips = t4_repo.list_slips_by_year(year)?;
        Ok(slips)
    })
}

/// File a T4 slip
#[tauri::command]
pub async fn file_t4_slip(
    slip_id: i64,
    filed_by: String,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        t4_repo.file_slip(slip_id, &filed_by)?;
        Ok(())
    })
}

/// Lock a T4 slip (archive)
#[tauri::command]
pub async fn lock_t4_slip(
    slip_id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        t4_repo.lock_slip(slip_id)?;
        Ok(())
    })
}

/// Unlock a T4 slip for amendment
#[tauri::command]
pub async fn unlock_t4_slip(
    slip_id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        t4_repo.unlock_slip(slip_id)?;
        Ok(())
    })
}

/// Get all years that have T4 slip data
#[tauri::command]
pub async fn get_t4_years(
    state: State<'_, AppState>,
) -> Result<Vec<i32>, CommandError> {
    with_db(&state, |db| {
        let years = db.t4().get_t4_years()
            .map_err(|e| CommandError::new(format!("Failed to list years: {}", e)))?;
        Ok(years)
    })
}

/// Get T4 slips for a year (read-only, from database)
/// Reads existing t4_slip records and returns them as T4 slip data
#[tauri::command]
pub async fn get_t4_slips_for_year(
    year: i32,
    state: State<'_, AppState>,
) -> Result<Vec<cpr_reports::t4::T4Data>, CommandError> {
    with_db(&state, |db| {
        let employees = db.employees().list_active()
            .map_err(|e| CommandError::new(format!("Failed to list employees: {}", e)))?;
        
        let t4_repo = db.t4();
        let mut t4_slips = Vec::new();
        
        for employee in employees.iter() {
            let slip = t4_repo.get_or_create_slip(employee.id.unwrap(), year)
                .map_err(|e| CommandError::new(format!("Failed to get T4 slip: {}", e)))?;
            
            let box_values = t4_repo.get_box_values(slip.id.unwrap())
                .map_err(|e| CommandError::new(format!("Failed to get box values: {}", e)))?;
            
            // Skip if no payroll data (all zeros)
            if box_values.is_empty() || box_values.iter().all(|bv| bv.final_value().is_zero()) {
                continue;
            }
            
            // Build T4Data from box values
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
            
            for bv in &box_values {
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
            
            // Computed net pay from box values for comparison
            let computed_net_pay = employment_income - cpp_contributions - cpp2_contributions
                - ei_premiums - income_tax_deducted - rpp_contributions;

            let t4_data = cpr_reports::t4::T4Data {
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
                net_pay: slip.net_pay,
                computed_net_pay,
            };
            
            t4_slips.push(t4_data);
        }
        
        Ok(t4_slips)
    })
}

/// Update T4 box values for a slip
#[tauri::command]
pub async fn update_t4_box_values(
    update: T4BoxValueUpdate,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let t4_repo = db.t4();
        
        // Map update fields to (String, Decimal) pairs
        let box_adjustments: Vec<(String, Decimal)> = vec![
            ("box_14".to_string(), update.box_14_adjustment),
            ("box_16".to_string(), update.box_16_adjustment),
            ("box_16a".to_string(), update.box_16a_adjustment),
            ("box_18".to_string(), update.box_18_adjustment),
            ("box_20".to_string(), update.box_20_adjustment),
            ("box_22".to_string(), update.box_22_adjustment),
            ("box_24".to_string(), update.box_24_adjustment),
            ("box_26".to_string(), update.box_26_adjustment),
            ("box_45".to_string(), update.box_45_adjustment),
            ("box_52".to_string(), update.box_52_adjustment),
        ];
        
        let slip_id = t4_repo.update_box_values(
            update.employee_id,
            update.year,
            &box_adjustments,
        )?;
        
        Ok(slip_id)
    })
}

