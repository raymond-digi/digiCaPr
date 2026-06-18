use tauri::State;
use crate::state::AppState;
use crate::error::CommandError;
use cpr_core::models::vacation::{
    VacationAccrual, VacationTimeOff, VacationTransactionType, VacationPayType,
    calculate_vacation_accrual, calculate_vacation_accrual_days, count_weekdays,
};
use cpr_core::models::PayType;

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

// =============================================
// Balance & History
// =============================================

/// Get vacation balance for an employee (fast O(1) lookup from employee record)
#[tauri::command]
pub async fn get_vacation_balance(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, CommandError> {
    with_db(&state, |db| {
        let balance = db.vacation().get_balance(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get balance: {}", e)))?;
        let balance_days = db.vacation().get_balance_days(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get days balance: {}", e)))?;
        let total_accrued = db.vacation().get_total_accrued(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get total accrued: {}", e)))?;
        let total_paid = db.vacation().get_total_paid(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get total paid: {}", e)))?;

        // Convert balance to cents for frontend
        let balance_cents = balance * rust_decimal::Decimal::from(100);

        Ok(serde_json::json!({
            "employee_id": employee_id,
            "balance": balance,
            "balance_cents": balance_cents,
            "balance_days": balance_days,
            "total_accrued": total_accrued,
            "total_paid": total_paid,
        }))
    })
}

/// Get transaction history for an employee
#[tauri::command]
pub async fn get_vacation_history(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<VacationAccrual>, CommandError> {
    with_db(&state, |db| {
        let history = db.vacation().get_history(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get history: {}", e)))?;
        Ok(history)
    })
}

// =============================================
// Accrual
// =============================================

/// Record vacation accrual from payroll (called when payroll is created/updated)
#[tauri::command]
pub async fn record_vacation_accrual(
    employee_id: i64,
    payroll_id: Option<i64>,
    gross_pay: f64,
    vacation_pay_rate: f64,
    state: State<'_, AppState>,
) -> Result<VacationAccrual, CommandError> {
    with_db(&state, |db| {
        let gross = rust_decimal::Decimal::try_from(gross_pay)
            .map_err(|e| CommandError::new(&format!("Invalid gross pay: {}", e)))?;
        let rate = rust_decimal::Decimal::try_from(vacation_pay_rate)
            .map_err(|e| CommandError::new(&format!("Invalid vacation pay rate: {}", e)))?;

        // Determine employee pay type
        let employee = db.employees().get(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get employee: {}", e)))?;

        // Hourly employees: accrue dollar value only (no days)
        // Non-hourly employees: accrue days only (no dollar value)
        let (accrual_amount, accrual_days) = match employee.pay_type {
            PayType::Hourly => {
                let dollar = calculate_vacation_accrual(gross, rate);
                (dollar, rust_decimal::Decimal::ZERO)
            }
            _ => {
                // Look up total_pay_periods from employee's payroll (default 26 for biweekly)
                let total_pay_periods = 26; // Default biweekly
                let days = calculate_vacation_accrual_days(rate, total_pay_periods);
                (rust_decimal::Decimal::ZERO, days)
            }
        };

        let current_balance = db.vacation().get_balance(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get balance: {}", e)))?;
        let new_balance = current_balance + accrual_amount;

        let current_balance_days = db.vacation().get_balance_days(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get days balance: {}", e)))?;
        let new_balance_days = current_balance_days + accrual_days;

        let mut transaction = VacationAccrual::new_dual(
            employee_id,
            chrono::Local::now().date_naive(),
            VacationTransactionType::Accrue,
            accrual_amount,
            new_balance,
            accrual_days,
            new_balance_days,
        );
        transaction.payroll_id = payroll_id;

        let _id = db.vacation().record_transaction(&mut transaction)
            .map_err(|e| CommandError::new(&format!("Failed to record accrual: {}", e)))?;

        Ok(transaction)
    })
}

// =============================================
// Adjustment
// =============================================

/// Record a manual vacation adjustment
/// Dollar amount and days are independent — both are saved as-is with no conversion.
///   - Hourly employees: only dollar amount is used, days are ignored
///   - Non-hourly employees: both dollar amount and days are saved independently
#[tauri::command]
pub async fn record_vacation_adjustment(
    employee_id: i64,
    amount: f64,
    amount_days: Option<f64>,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<VacationAccrual, CommandError> {
    with_db(&state, |db| {
        let employee = db.employees().get(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get employee: {}", e)))?;

        // Dollar amount — always saved
        let adj_amount = rust_decimal::Decimal::try_from(amount)
            .map_err(|e| CommandError::new(&format!("Invalid adjustment amount: {}", e)))?;

        // Days amount — only for non-hourly employees, saved as-is (no conversion)
        let adj_days = match employee.pay_type {
            PayType::Hourly => rust_decimal::Decimal::ZERO,
            _ => {
                if let Some(days_val) = amount_days {
                    rust_decimal::Decimal::try_from(days_val)
                        .map_err(|e| CommandError::new(&format!("Invalid adjustment days: {}", e)))?
                } else {
                    rust_decimal::Decimal::ZERO
                }
            }
        };

        let current_balance = db.vacation().get_balance(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get balance: {}", e)))?;
        let new_balance = current_balance + adj_amount;

        let new_balance_days = match employee.pay_type {
            PayType::Hourly => rust_decimal::Decimal::ZERO,
            _ => {
                let current_balance_days = db.vacation().get_balance_days(employee_id)
                    .map_err(|e| CommandError::new(&format!("Failed to get days balance: {}", e)))?;
                current_balance_days + adj_days
            }
        };

        let mut transaction = VacationAccrual::new_dual(
            employee_id,
            chrono::Local::now().date_naive(),
            VacationTransactionType::Adjust,
            adj_amount,
            new_balance,
            adj_days,
            new_balance_days,
        );
        transaction.notes = notes;

        let _ = db.vacation().record_transaction(&mut transaction)
            .map_err(|e| CommandError::new(&format!("Failed to record adjustment: {}", e)))?;

        Ok(transaction)
    })
}

// =============================================
// Time Off
// =============================================

/// Create a vacation time off record
/// pay_type is auto-determined by the employee's pay_type:
///   - Hourly employee → unpaid (payout_amount = 0)
///   - Non-hourly employee → paid (payout_amount auto-calculated from pay rate)
#[tauri::command]
pub async fn create_vacation_time_off(
    employee_id: i64,
    start_date: String,
    end_date: String,
    estimated_payout: f64,
    payout_amount: f64,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<VacationTimeOff, CommandError> {
    with_db(&state, |db| {
        let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(&format!("Invalid start date: {}", e)))?;
        let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(&format!("Invalid end date: {}", e)))?;
        let initial_payout = rust_decimal::Decimal::try_from(estimated_payout)
            .map_err(|e| CommandError::new(&format!("Invalid initial payout amount: {}", e)))?;
        let payout = rust_decimal::Decimal::try_from(payout_amount)
            .map_err(|e| CommandError::new(&format!("Invalid payout amount: {}", e)))?;

        // Look up employee to determine pay_type
        let employee = db.employees().get(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get employee: {}", e)))?;

        let pay_type = match employee.pay_type {
            PayType::Hourly => VacationPayType::Unpaid,
            _ => VacationPayType::Paid,
        };

        // Calculate days taken for non-hourly employees
        let weekdays = count_weekdays(start, end);
        let days_taken = match employee.pay_type {
            PayType::Hourly => None,
            _ => Some(rust_decimal::Decimal::from(weekdays)),
        };

        let mut time_off = VacationTimeOff::new(
            employee_id,
            start,
            end,
            pay_type,
            initial_payout,  // estimated_payout = auto-calculated estimate (readonly)
            payout,          // payout_amount = user's value (editable)
        );
        time_off.days_taken = days_taken;
        time_off.notes = notes;

        let id = db.vacation().create_time_off(&mut time_off)
            .map_err(|e| CommandError::new(&format!("Failed to create time off: {}", e)))?;

        // Create an accrual transaction for paid time off:
        //   - Hourly employees: only when payout > 0 (dollar-based)
        //   - Non-hourly employees: always (days-based, even when dollar payout is 0)
        let should_create_accrual = match employee.pay_type {
            PayType::Hourly => payout != rust_decimal::Decimal::ZERO,
            _ => true, // Non-hourly always create accrual to track days
        };
        if should_create_accrual {
            let current_balance = db.vacation().get_balance(employee_id)
                .map_err(|e| CommandError::new(&format!("Failed to get balance: {}", e)))?;
            let new_balance = current_balance - payout.abs();

            // For non-hourly: deduct days from day balance
            let days_deducted = rust_decimal::Decimal::from(weekdays);
            let current_balance_days = db.vacation().get_balance_days(employee_id)
                .map_err(|e| CommandError::new(&format!("Failed to get days balance: {}", e)))?;
            let new_balance_days = current_balance_days - days_deducted;

            let mut accrual = VacationAccrual::new_dual(
                employee_id,
                start,
                VacationTransactionType::Timeoff,
                -payout.abs(),
                new_balance,
                -days_deducted,
                new_balance_days,
            );

            let accrual_id = db.vacation().record_transaction(&mut accrual)
                .map_err(|e| CommandError::new(&format!("Failed to record accrual: {}", e)))?;

            // Link the accrual transaction to the time off record
            time_off.vacation_accrual_id = Some(accrual_id);
            db.vacation().update_time_off_accrual_id(id, Some(accrual_id))
                .map_err(|e| CommandError::new(&format!("Failed to link accrual: {}", e)))?;

            time_off.id = Some(id);
        }

        Ok(time_off)
    })
}

/// Update a vacation time off record
/// If payout_amount changes and there's a linked accrual, update the accrual too
#[tauri::command]
pub async fn update_vacation_time_off(
    time_off_id: i64,
    start_date: String,
    end_date: String,
    payout_amount: f64,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<VacationTimeOff, CommandError> {
    with_db(&state, |db| {
        let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(&format!("Invalid start date: {}", e)))?;
        let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
            .map_err(|e| CommandError::new(&format!("Invalid end date: {}", e)))?;
        let new_payout = rust_decimal::Decimal::try_from(payout_amount)
            .map_err(|e| CommandError::new(&format!("Invalid payout amount: {}", e)))?;

        // Get the existing time off record
        let mut time_off = db.vacation().get_time_off(time_off_id)
            .map_err(|e| CommandError::new(&format!("Failed to get time off: {}", e)))?;

        let old_payout = time_off.payout_amount;

        // Update the time off record
        time_off.start_date = start;
        time_off.end_date = end;
        time_off.payout_amount = new_payout;
        time_off.notes = notes;

        db.vacation().update_time_off(&time_off)
            .map_err(|e| CommandError::new(&format!("Failed to update time off: {}", e)))?;

        // If payout changed and there's a linked accrual, update the accrual transaction
        if new_payout != old_payout {
            if let Some(accrual_id) = time_off.vacation_accrual_id {
                // Get the current balance
                let current_balance = db.vacation().get_balance(time_off.employee_id)
                    .map_err(|e| CommandError::new(&format!("Failed to get balance: {}", e)))?;

                // Calculate new balance: reverse old payout effect, apply new payout
                let new_balance = current_balance + old_payout - new_payout;

                let mut accrual = VacationAccrual::new(
                    time_off.employee_id,
                    time_off.start_date,
                    VacationTransactionType::Timeoff,
                    -new_payout.abs(),
                    new_balance,
                );
                accrual.id = Some(accrual_id);

                db.vacation().update_transaction(&accrual)
                    .map_err(|e| CommandError::new(&format!("Failed to update accrual: {}", e)))?;
            }
        }

        Ok(time_off)
    })
}

/// Delete a vacation time off record
/// Also deletes the linked accrual transaction if present
#[tauri::command]
pub async fn delete_vacation_time_off(
    time_off_id: i64,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    with_db(&state, |db| {
        // Get the time off record to find the linked accrual
        let time_off = db.vacation().get_time_off(time_off_id)
            .map_err(|e| CommandError::new(&format!("Failed to get time off: {}", e)))?;

        // Delete the linked accrual transaction first (reverses balance effect)
        if let Some(accrual_id) = time_off.vacation_accrual_id {
            db.vacation().delete_transaction(accrual_id)
                .map_err(|e| CommandError::new(&format!("Failed to delete accrual: {}", e)))?;
        }

        // Delete the time off record
        db.vacation().delete_time_off(time_off_id)
            .map_err(|e| CommandError::new(&format!("Failed to delete time off: {}", e)))?;

        Ok(())
    })
}

/// Get all time off records for an employee
#[tauri::command]
pub async fn get_vacation_time_off_history(
    employee_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<VacationTimeOff>, CommandError> {
    with_db(&state, |db| {
        let history = db.vacation().get_time_off_history(employee_id)
            .map_err(|e| CommandError::new(&format!("Failed to get time off history: {}", e)))?;
        Ok(history)
    })
}
