use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use rust_decimal::Decimal;
use cpr_core::models::vacation::{
    VacationAccrual, VacationTimeOff, VacationTransactionType, VacationPayType,
};
use crate::currency::{decimal_to_cents, decimal_to_cents_signed};
use crate::DbResult;

pub struct VacationRepository {
    conn: Arc<Mutex<Connection>>,
}

impl VacationRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // =============================================
    // Accrual Transactions
    // =============================================

    /// Record a vacation accrual transaction and update employee balance
    pub fn record_transaction(
        &self,
        transaction: &mut VacationAccrual,
    ) -> DbResult<i64> {
        transaction.validate().map_err(|e| crate::DbError::InvalidData(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        // Convert days to thousandths for DB storage
        let amount_days_thousandths = decimal_to_days_thousandths(&transaction.amount_days);
        let balance_after_days_thousandths = decimal_to_days_thousandths(&transaction.balance_after_days);

        conn.execute(
            "INSERT INTO vacation_accrual (
                employee_id, accrual_date, payroll_id, transaction_type,
                amount, amount_days, balance_after, balance_after_days, notes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                transaction.employee_id,
                transaction.accrual_date.format("%Y-%m-%d").to_string(),
                transaction.payroll_id,
                transaction.transaction_type.as_str(),
                decimal_to_cents_signed(&transaction.amount).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                amount_days_thousandths,
                decimal_to_cents_signed(&transaction.balance_after).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                balance_after_days_thousandths,
                transaction.notes,
                transaction.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        transaction.id = Some(id);

        // Update employee vacation_balance and vacation_balance_days
        conn.execute(
            "UPDATE employee SET vacation_balance = ?1, vacation_balance_days = ?2 WHERE id = ?3",
            params![
                decimal_to_cents_signed(&transaction.balance_after).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                balance_after_days_thousandths,
                transaction.employee_id,
            ],
        )?;

        Ok(id)
    }

    /// Update an existing accrual transaction and adjust employee balance
    pub fn update_transaction(
        &self,
        transaction: &VacationAccrual,
    ) -> DbResult<()> {
        let id = transaction.id.ok_or(crate::DbError::InvalidData(
            "Transaction must have an id to update".to_string()
        ))?;

        let conn = self.conn.lock().unwrap();

        // First, get the old balance_after to compute the difference
        let old_balance_cents: i64 = conn.query_row(
            "SELECT balance_after FROM vacation_accrual WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        let old_balance = Decimal::new(old_balance_cents, 2);
        let balance_diff = transaction.balance_after - old_balance;

        // Get old days balance
        let old_balance_days_thousandths: i64 = conn.query_row(
            "SELECT balance_after_days FROM vacation_accrual WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        let old_balance_days = Decimal::new(old_balance_days_thousandths, 3);
        let balance_days_diff = transaction.balance_after_days - old_balance_days;

        let amount_days_thousandths = decimal_to_days_thousandths(&transaction.amount_days);
        let balance_after_days_thousandths = decimal_to_days_thousandths(&transaction.balance_after_days);

        conn.execute(
            "UPDATE vacation_accrual SET amount = ?1, amount_days = ?2, balance_after = ?3, balance_after_days = ?4, notes = ?5 WHERE id = ?6",
            params![
                decimal_to_cents_signed(&transaction.amount).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                amount_days_thousandths,
                decimal_to_cents_signed(&transaction.balance_after).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                balance_after_days_thousandths,
                transaction.notes,
                id,
            ],
        )?;

        // Adjust employee vacation_balance and vacation_balance_days by the difference
        if balance_diff != Decimal::ZERO || balance_days_diff != Decimal::ZERO {
            conn.execute(
                "UPDATE employee SET vacation_balance = vacation_balance + ?1, vacation_balance_days = vacation_balance_days + ?2 WHERE id = ?3",
                params![
                    decimal_to_cents_signed(&balance_diff).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                    decimal_to_days_thousandths(&balance_days_diff),
                    transaction.employee_id,
                ],
            )?;
        }

        Ok(())
    }

    /// Delete an accrual transaction and reverse its effect on employee balance
    pub fn delete_transaction(&self, transaction_id: i64) -> DbResult<VacationAccrual> {
        let conn = self.conn.lock().unwrap();

        // Read the transaction before deleting
        let type_str: String = conn.query_row(
            "SELECT transaction_type FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        ).map_err(|_| crate::DbError::NotFound(format!("Accrual transaction {} not found", transaction_id)))?;

        let employee_id: i64 = conn.query_row(
            "SELECT employee_id FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )?;

        let balance_after_cents: i64 = conn.query_row(
            "SELECT balance_after FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )?;
        let balance_after = Decimal::new(balance_after_cents, 2);

        let amount_cents: i64 = conn.query_row(
            "SELECT amount FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )?;
        let amount = Decimal::new(amount_cents, 2);

        let balance_after_days_thousandths: i64 = conn.query_row(
            "SELECT balance_after_days FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )?;
        let balance_after_days = Decimal::new(balance_after_days_thousandths, 3);

        let amount_days_thousandths: i64 = conn.query_row(
            "SELECT amount_days FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )?;
        let amount_days = Decimal::new(amount_days_thousandths, 3);

        let transaction_type = VacationTransactionType::from_str(&type_str)
            .ok_or(crate::DbError::InvalidData(format!("Invalid transaction type: {}", type_str)))?;

        conn.execute(
            "DELETE FROM vacation_accrual WHERE id = ?1",
            [transaction_id],
        )?;

        // Reverse the balance effect (both dollar and day balances)
        let balance_diff = -amount;
        let balance_days_diff = -amount_days;
        if balance_diff != Decimal::ZERO || balance_days_diff != Decimal::ZERO {
            conn.execute(
                "UPDATE employee SET vacation_balance = vacation_balance + ?1, vacation_balance_days = vacation_balance_days + ?2 WHERE id = ?3",
                params![
                    decimal_to_cents_signed(&balance_diff).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                    decimal_to_days_thousandths(&balance_days_diff),
                    employee_id,
                ],
            )?;
        }

        Ok(VacationAccrual {
            id: Some(transaction_id),
            employee_id,
            accrual_date: chrono::NaiveDate::default(),
            payroll_id: None,
            transaction_type,
            amount,
            balance_after,
            amount_days,
            balance_after_days,
            notes: None,
            created_at: chrono::Utc::now(),
        })
    }

    /// Get current vacation balance from employee record (fast O(1) lookup)
    pub fn get_balance(&self, employee_id: i64) -> DbResult<Decimal> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT vacation_balance FROM employee WHERE id = ?1"
        )?;

        let balance_cents: i64 = stmt.query_row([employee_id], |row| {
            row.get(0)
        })?;

        Ok(Decimal::new(balance_cents, 2))
    }

    /// Get current vacation balance in days from employee record (fast O(1) lookup)
    pub fn get_balance_days(&self, employee_id: i64) -> DbResult<Decimal> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT vacation_balance_days FROM employee WHERE id = ?1"
        )?;

        let balance_thousandths: i64 = stmt.query_row([employee_id], |row| {
            row.get(0)
        })?;

        Ok(Decimal::new(balance_thousandths, 3))
    }

    /// Get both dollar and day balances for an employee
    pub fn get_both_balances(&self, employee_id: i64) -> DbResult<(Decimal, Decimal)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT vacation_balance, vacation_balance_days FROM employee WHERE id = ?1"
        )?;

        let (balance_cents, balance_days_thousandths) = stmt.query_row([employee_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        Ok((Decimal::new(balance_cents, 2), Decimal::new(balance_days_thousandths, 3)))
    }

    /// Get transaction history for an employee
    pub fn get_history(&self, employee_id: i64) -> DbResult<Vec<VacationAccrual>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, accrual_date, payroll_id, transaction_type,
                    amount, amount_days, balance_after, balance_after_days, notes, created_at
             FROM vacation_accrual
             WHERE employee_id = ?1
             ORDER BY accrual_date DESC, id DESC"
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            let type_str: String = row.get(4)?;
            let transaction_type = VacationTransactionType::from_str(&type_str)
                .ok_or(rusqlite::Error::InvalidQuery)?;
            let amount_cents: i64 = row.get(5)?;
            let amount_days_thousandths: i64 = row.get(6)?;
            let balance_cents: i64 = row.get(7)?;
            let balance_days_thousandths: i64 = row.get(8)?;

            Ok(VacationAccrual {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                accrual_date: row.get(2)?,
                payroll_id: row.get(3)?,
                transaction_type,
                amount: Decimal::new(amount_cents, 2),
                amount_days: Decimal::new(amount_days_thousandths, 3),
                balance_after: Decimal::new(balance_cents, 2),
                balance_after_days: Decimal::new(balance_days_thousandths, 3),
                notes: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    /// Get total accrued amount for an employee
    pub fn get_total_accrued(&self, employee_id: i64) -> DbResult<Decimal> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(amount), 0) FROM vacation_accrual
             WHERE employee_id = ?1 AND transaction_type = 'accrue'"
        )?;

        let total_cents: i64 = stmt.query_row([employee_id], |row| {
            row.get(0)
        })?;

        Ok(Decimal::new(total_cents, 2))
    }

    /// Get total paid out for an employee
    pub fn get_total_paid(&self, employee_id: i64) -> DbResult<Decimal> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT COALESCE(SUM(ABS(amount)), 0) FROM vacation_accrual
             WHERE employee_id = ?1 AND transaction_type IN ('payout', 'timeoff')"
        )?;

        let total_cents: i64 = stmt.query_row([employee_id], |row| {
            row.get(0)
        })?;

        Ok(Decimal::new(total_cents, 2))
    }

    // =============================================
    // Time Off
    // =============================================

    /// Create a vacation time off record
    pub fn create_time_off(&self, time_off: &mut VacationTimeOff) -> DbResult<i64> {
        time_off.validate().map_err(|e| crate::DbError::InvalidData(e.to_string()))?;

        let conn = self.conn.lock().unwrap();

        let days_taken_thousandths = decimal_to_days_thousandths(&time_off.days_taken.unwrap_or(Decimal::ZERO));

        conn.execute(
            "INSERT INTO vacation_time_off (
                employee_id, vacation_accrual_id, start_date, end_date,
                pay_type, estimated_payout, payout_amount, days_taken, notes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                time_off.employee_id,
                time_off.vacation_accrual_id,
                time_off.start_date.format("%Y-%m-%d").to_string(),
                time_off.end_date.format("%Y-%m-%d").to_string(),
                time_off.pay_type.as_str(),
                decimal_to_cents(&time_off.estimated_payout).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                decimal_to_cents(&time_off.payout_amount).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                days_taken_thousandths,
                time_off.notes,
                time_off.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        time_off.id = Some(id);
        Ok(id)
    }

    /// Get all time off records for an employee
    pub fn get_time_off_history(&self, employee_id: i64) -> DbResult<Vec<VacationTimeOff>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, vacation_accrual_id, start_date, end_date,
                    pay_type, estimated_payout, payout_amount, days_taken, notes, created_at
             FROM vacation_time_off
             WHERE employee_id = ?1
             ORDER BY start_date DESC"
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            let pay_type_str: String = row.get(5)?;
            let initial_payout_cents: i64 = row.get(6)?;
            let payout_cents: i64 = row.get(7)?;
            let days_taken_thousandths: i64 = row.get(8)?;

            Ok(VacationTimeOff {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                vacation_accrual_id: row.get(2)?,
                start_date: row.get(3)?,
                end_date: row.get(4)?,
                pay_type: VacationPayType::from_str(&pay_type_str)
                    .ok_or(rusqlite::Error::InvalidQuery)?,
                estimated_payout: Decimal::new(initial_payout_cents, 2),
                payout_amount: Decimal::new(payout_cents, 2),
                days_taken: Some(Decimal::new(days_taken_thousandths, 3)),
                notes: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    /// Update a time off record
    pub fn update_time_off(&self, time_off: &VacationTimeOff) -> DbResult<()> {
        let id = time_off.id.ok_or(crate::DbError::InvalidData(
            "Time off must have an id to update".to_string()
        ))?;

        let conn = self.conn.lock().unwrap();
        let days_taken_thousandths = decimal_to_days_thousandths(&time_off.days_taken.unwrap_or(Decimal::ZERO));

        let rows_affected = conn.execute(
            "UPDATE vacation_time_off SET
                start_date = ?1, end_date = ?2, payout_amount = ?3, days_taken = ?4, notes = ?5
             WHERE id = ?6",
            params![
                time_off.start_date.format("%Y-%m-%d").to_string(),
                time_off.end_date.format("%Y-%m-%d").to_string(),
                decimal_to_cents(&time_off.payout_amount).map_err(|e| crate::DbError::InvalidData(e.to_string()))?,
                days_taken_thousandths,
                time_off.notes,
                id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(crate::DbError::NotFound(
                format!("Time off record {} not found", id)
            ));
        }

        Ok(())
    }

    /// Update the vacation_accrual_id for a time off record
    pub fn update_time_off_accrual_id(&self, time_off_id: i64, accrual_id: Option<i64>) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE vacation_time_off SET vacation_accrual_id = ?1 WHERE id = ?2",
            params![accrual_id, time_off_id],
        )?;
        Ok(())
    }

    /// Delete a time off record
    pub fn delete_time_off(&self, time_off_id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM vacation_time_off WHERE id = ?1",
            [time_off_id],
        )?;

        if rows_affected == 0 {
            return Err(crate::DbError::NotFound(
                format!("Time off record {} not found", time_off_id)
            ));
        }

        Ok(())
    }

    /// Get a single time off record by id
    pub fn get_time_off(&self, time_off_id: i64) -> DbResult<VacationTimeOff> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, employee_id, vacation_accrual_id, start_date, end_date,
                    pay_type, estimated_payout, payout_amount, days_taken, notes, created_at
             FROM vacation_time_off WHERE id = ?1",
            [time_off_id],
            |row| {
                let pay_type_str: String = row.get(5)?;
                let initial_payout_cents: i64 = row.get(6)?;
                let payout_cents: i64 = row.get(7)?;
                let days_taken_thousandths: i64 = row.get(8)?;

                Ok(VacationTimeOff {
                    id: row.get(0)?,
                    employee_id: row.get(1)?,
                    vacation_accrual_id: row.get(2)?,
                    start_date: row.get(3)?,
                    end_date: row.get(4)?,
                    pay_type: VacationPayType::from_str(&pay_type_str)
                        .ok_or(rusqlite::Error::InvalidQuery)?,
                    estimated_payout: Decimal::new(initial_payout_cents, 2),
                    payout_amount: Decimal::new(payout_cents, 2),
                    days_taken: Some(Decimal::new(days_taken_thousandths, 3)),
                    notes: row.get(9)?,
                    created_at: row.get(10)?,
                })
            },
        ).map_err(|e| crate::DbError::NotFound(format!("Time off record {} not found: {}", time_off_id, e)))
    }
}

/// Helper: Convert a Decimal days value to thousandths integer for DB storage
/// E.g., 1.5 days → 1500, 0.3846 days → 385
fn decimal_to_days_thousandths(value: &Decimal) -> i64 {
    (value * rust_decimal::Decimal::from(1000)).round_dp(0).to_string().parse::<i64>().unwrap_or(0)
}
