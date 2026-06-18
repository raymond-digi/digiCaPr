use tauri::State;
use crate::state::AppState;
use crate::error::CommandError;
use chrono::{NaiveDate, Utc, Datelike};
use rust_decimal::Decimal;


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

/// Get remittance summary for unfiled history payrolls before a cutoff date.
/// Only includes history payrolls with remittance_id IS NULL (prevents double-counting).
/// Current payroll drafts are excluded — only posted history payrolls are remitted.
#[tauri::command]
pub async fn get_remittance_summary(
    cutoff_date: String,
    state: State<'_, AppState>,
) -> Result<RemittanceSummary, CommandError> {
    with_db(&state, |db| {
        let cutoff = NaiveDate::parse_from_str(&cutoff_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        // Only query history payrolls that haven't been remitted yet
        let year = cutoff.year();
        let start_date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let paid_payrolls = db.payroll_history().list_unremitted_by_date_range(start_date, cutoff)?;

        if paid_payrolls.is_empty() {
            return Ok(RemittanceSummary {
                unfiled_payrolls_count: 0,
                total_earnings: Decimal::ZERO,
                total_cpp: Decimal::ZERO,
                total_cpp2: Decimal::ZERO,
                total_ei: Decimal::ZERO,
                total_federal_tax: Decimal::ZERO,
                total_provincial_tax: Decimal::ZERO,
                grand_total: Decimal::ZERO,
                period_start: cutoff.format("%Y-%m-%d").to_string(),
                period_end: cutoff.format("%Y-%m-%d").to_string(),
            });
        }

        let mut total_earnings = Decimal::ZERO;
        let mut total_cpp = Decimal::ZERO;
        let mut total_cpp2 = Decimal::ZERO;
        let mut total_ei = Decimal::ZERO;
        let mut total_federal_tax = Decimal::ZERO;
        let mut total_provincial_tax = Decimal::ZERO;

        let mut period_start = paid_payrolls[0].pay_period_start;
        let mut period_end = paid_payrolls[0].pay_period_end;

        for payroll in &paid_payrolls {
            total_earnings += payroll.gross_pay;
            // CPP: employee contribution + employer match = 2x employee CPP
            total_cpp += payroll.deductions.cpp * Decimal::TWO;
            // CPP2: employee contribution + employer match = 2x employee CPP2
            total_cpp2 += payroll.deductions.cpp2 * Decimal::TWO;
            // EI: employee premium + employer premium = 1.4x employee EI (total 2.4x)
            total_ei += payroll.deductions.ei * Decimal::new(24, 1);
            total_federal_tax += payroll.deductions.federal_tax;
            total_provincial_tax += payroll.deductions.provincial_tax;

            if payroll.pay_period_start < period_start {
                period_start = payroll.pay_period_start;
            }
            if payroll.pay_period_end > period_end {
                period_end = payroll.pay_period_end;
            }
        }

        let grand_total = total_cpp + total_cpp2 + total_ei + total_federal_tax + total_provincial_tax;

        Ok(RemittanceSummary {
            unfiled_payrolls_count: paid_payrolls.len(),
            total_earnings,
            total_cpp,
            total_cpp2,
            total_ei,
            total_federal_tax,
            total_provincial_tax,
            grand_total,
            period_start: period_start.format("%Y-%m-%d").to_string(),
            period_end: period_end.format("%Y-%m-%d").to_string(),
        })
    })
}

/// Create remittance from summary and link included payrolls.
/// After creating the remittance record, all matching history payrolls
/// (pay_date <= cutoff, remittance_id IS NULL) are linked to prevent double-counting.
#[tauri::command]
pub async fn create_remittance(
    cutoff_date: String,
    cra_confirmation: Option<String>,
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    with_db(&state, |db| {
        let cutoff = NaiveDate::parse_from_str(&cutoff_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(format!("Invalid date format: {}", e)))?;

        // Only query history payrolls that haven't been remitted yet
        let year = cutoff.year();
        let start_date = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let paid_payrolls = db.payroll_history().list_unremitted_by_date_range(start_date, cutoff)?;

        if paid_payrolls.is_empty() {
            return Err(CommandError::new("No unfiled payrolls found"));
        }

        let mut total_earnings = Decimal::ZERO;
        let mut total_cpp = Decimal::ZERO;
        let mut total_cpp2 = Decimal::ZERO;
        let mut total_ei = Decimal::ZERO;
        let mut total_federal_tax = Decimal::ZERO;
        let mut total_provincial_tax = Decimal::ZERO;

        let mut period_start = paid_payrolls[0].pay_period_start;
        let mut period_end = paid_payrolls[0].pay_period_end;

        for payroll in &paid_payrolls {
            total_earnings += payroll.gross_pay;
            // CPP: employee contribution + employer match = 2x employee CPP
            total_cpp += payroll.deductions.cpp * Decimal::TWO;
            // CPP2: employee contribution + employer match = 2x employee CPP2
            total_cpp2 += payroll.deductions.cpp2 * Decimal::TWO;
            // EI: employee premium + employer premium = 1.4x employee EI (total 2.4x)
            total_ei += payroll.deductions.ei * Decimal::new(24, 1);
            total_federal_tax += payroll.deductions.federal_tax;
            total_provincial_tax += payroll.deductions.provincial_tax;

            if payroll.pay_period_start < period_start {
                period_start = payroll.pay_period_start;
            }
            if payroll.pay_period_end > period_end {
                period_end = payroll.pay_period_end;
            }
        }

        let grand_total = total_cpp + total_cpp2 + total_ei + total_federal_tax + total_provincial_tax;

        // Count unique employees
        let total_employees: i64 = paid_payrolls.iter()
            .map(|p| p.employee_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let mut remittance = cpr_db::repository::remittance::Remittance {
            id: None,
            period_start,
            period_end,
            total_employees,
            total_earnings,
            total_cpp,
            total_cpp2,
            total_ei,
            total_federal_tax,
            total_provincial_tax,
            grand_total,
            cra_report_reference: cra_confirmation,
            generated_at: Utc::now(),
        };

        let id = db.remittance().create(&mut remittance)?;

        // Link all included history payrolls to this remittance to prevent double-counting
        db.payroll_history().link_to_remittance(id, cutoff)?;

        Ok(id)
    })
}

/// Get available years for remittance history
#[tauri::command]
pub async fn get_remittance_years(
    state: State<'_, AppState>,
) -> Result<Vec<i32>, CommandError> {
    with_db(&state, |db| {
        let years = db.remittance().get_available_years()?;
        Ok(years)
    })
}

/// List remittances, optionally filtered by year
#[tauri::command]
pub async fn list_remittances(
    year: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<cpr_db::repository::remittance::Remittance>, CommandError> {
    with_db(&state, |db| {
        let target_year = year.unwrap_or_else(|| chrono::Utc::now().date_naive().year());
        let records = db.remittance().list_by_year(target_year)?;
        Ok(records)
    })
}

/// Get a single remittance
#[tauri::command]
pub async fn get_remittance(
    id: i64,
    state: State<'_, AppState>,
) -> Result<cpr_db::repository::remittance::Remittance, CommandError> {
    with_db(&state, |db| {
        let remittance = db.remittance().get(id)?;
        Ok(remittance)
    })
}

/// Delete a remittance and unlink its history payrolls
#[tauri::command]
pub async fn delete_remittance(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        // Unlink history payrolls before deleting the remittance
        let history = db.payroll_history();
        history.unlink_from_remittance(id)?;

        db.remittance().delete(id)?;
        Ok(())
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RemittanceSummary {
    pub unfiled_payrolls_count: usize,
    pub total_earnings: Decimal,
    pub total_cpp: Decimal,
    pub total_cpp2: Decimal,
    pub total_ei: Decimal,
    pub total_federal_tax: Decimal,
    pub total_provincial_tax: Decimal,
    pub grand_total: Decimal,
    pub period_start: String,
    pub period_end: String,
}
