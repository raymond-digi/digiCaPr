use crate::currency::{cents_to_decimal, cents_to_decimal_signed, convert_cents, decimal_to_cents, decimal_to_cents_signed};
use crate::utils::*;
use rusqlite::{params, types::Type, Connection};
use std::sync::{Arc, Mutex};

use crate::{DbError, DbResult};
use cpr_core::models::{Address, Employee, PayType};

/// Create a descriptive rusqlite error that includes employee context.
/// This ensures the user sees WHICH employee has the data issue and WHAT the value is.
fn employee_data_error(emp_num: &str, first: &str, last: &str, field: &str, detail: impl std::fmt::Display) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Employee #{} ({} {}): {} is invalid — {}",
                emp_num, first, last, field, detail
            ),
        )),
    )
}


pub struct EmployeeRepository {
    conn: Arc<Mutex<Connection>>,
}

impl EmployeeRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, employee: &mut Employee) -> DbResult<i64> {
        employee.validate()?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO employee (
                employee_number, first_name, last_name, notes, sin,
                address_street, address_city, address_province, address_postal_code,
                pay_type, pay_rate,
                date_of_birth, vacation_pay_rate, vacation_balance, vacation_balance_days, overtime_multiplier,
                ei_exempt, cpp_exempt,
                hire_date, hire_province, termination_date, dental_benefit, is_active, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
            params![
                employee.employee_number,
                employee.first_name,
                employee.last_name,
                employee.notes.as_ref().map(|s| s.as_str()),
                employee.sin,
                employee.address.street,
                employee.address.city,
                employee.address.province,
                employee.address.postal_code,
                employee.pay_type.as_str(),
                decimal_to_cents(&employee.pay_rate)?,
                employee.date_of_birth.format("%Y-%m-%d").to_string(),
                decimal_to_basis_points(&employee.vacation_pay_rate),
                decimal_to_cents_signed(&employee.vacation_balance)
                    .map_err(|e| DbError::InvalidData(format!(
                        "Employee #{} ({} {}): vacation_balance is invalid — {} (value: {})",
                        employee.employee_number, employee.first_name, employee.last_name,
                        e, employee.vacation_balance
                    )))?,
                decimal_to_cents_signed(&employee.vacation_balance_days)
                    .map_err(|e| DbError::InvalidData(format!(
                        "Employee #{} ({} {}): vacation_balance_days is invalid — {} (value: {})",
                        employee.employee_number, employee.first_name, employee.last_name,
                        e, employee.vacation_balance_days
                    )))?,
                decimal_to_basis_points(&employee.overtime_multiplier),
                employee.ei_exempt as i32,
                employee.cpp_exempt as i32,
                employee.hire_date.format("%Y-%m-%d").to_string(),
                employee.hire_province,
                employee.termination_date.map(|d| d.format("%Y-%m-%d").to_string()),
                employee.dental_benefit,
                employee.is_active as i32,
                employee.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        employee.id = Some(id);
        Ok(id)
    }

    pub fn get(&self, id: i64) -> DbResult<Employee> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_number, first_name, last_name, notes, sin,
                    address_street, address_city, address_province, address_postal_code,
                    pay_type, pay_rate,
                    date_of_birth, vacation_pay_rate, vacation_balance, vacation_balance_days, overtime_multiplier,
                    ei_exempt, cpp_exempt,
                    hire_date, hire_province, termination_date, dental_benefit, is_active, created_at
             FROM employee WHERE id = ?1",
        )?;

        let employee = stmt.query_row([id], |row| {
            // Read identity fields first for error context
            let employee_number: String = row.get(1)?;
            let first_name: String = row.get(2)?;
            let last_name: String = row.get(3)?;

            let pay_type_str: String = row.get(10)?;
            let pay_type = PayType::from_str(&pay_type_str)
                .ok_or_else(|| employee_data_error(&employee_number, &first_name, &last_name, "pay_type", format!("unknown type '{}' (expected Hourly, Weekly, Monthly, or Annual)", pay_type_str)))?;

            let pay_rate_cents: i64 = row.get(11)?;
            let pay_rate = cents_to_decimal(pay_rate_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "pay_rate", format!("{} (stored as {} cents)", e, pay_rate_cents)))?;

            let vacation_balance_cents: i64 = row.get(14)?;
            let vacation_balance = cents_to_decimal_signed(vacation_balance_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "vacation_balance", format!("{} (stored as {} cents)", e, vacation_balance_cents)))?;

            let vacation_balance_days_cents: i64 = row.get(15)?;
            let vacation_balance_days = cents_to_decimal_signed(vacation_balance_days_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "vacation_balance_days", format!("{} (stored as {} cents)", e, vacation_balance_days_cents)))?;

            Ok(Employee {
                id: Some(row.get(0)?),
                employee_number,
                first_name,
                last_name,
                notes: row.get::<_, Option<String>>(4)?,
                sin: row.get(5)?,
                address: Address { street: row.get(6)?, city: row.get(7)?, province: row.get(8)?, postal_code: row.get(9)? },
                pay_type,
                pay_rate,
                date_of_birth: row.get(12)?,
                vacation_pay_rate: basis_points_to_decimal(row.get::<_, i64>(13)?),
                vacation_balance,
                vacation_balance_days,
                overtime_multiplier: basis_points_to_decimal(row.get::<_, i64>(16)?),
                ei_exempt: row.get::<_, i32>(17)? != 0,
                cpp_exempt: row.get::<_, i32>(18)? != 0,
                hire_date: row.get(19)?,
                hire_province: row.get(20)?,
                termination_date: row.get(21)?,
                dental_benefit: row.get::<_, i32>(22)?,
                is_active: row.get::<_, i32>(23)? != 0,
                created_at: row.get(24)?,
            })
        })?;

        Ok(employee)
    }

    pub fn update(&self, employee: &Employee) -> DbResult<()> {
        employee.validate()?;

        let id = employee.id.ok_or_else(|| DbError::InvalidData("Employee ID is required for update".to_string()))?;

        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE employee SET
                employee_number = ?1, first_name = ?2, last_name = ?3, notes = ?4, sin = ?5,
                address_street = ?6, address_city = ?7, address_province = ?8, address_postal_code = ?9,
                pay_type = ?10, pay_rate = ?11,
                date_of_birth = ?12, vacation_pay_rate = ?13, vacation_balance = ?14, vacation_balance_days = ?15, overtime_multiplier = ?16,
                ei_exempt = ?17, cpp_exempt = ?18,
                hire_date = ?19, hire_province = ?20, termination_date = ?21, dental_benefit = ?22, is_active = ?23
             WHERE id = ?24",
            params![
                employee.employee_number,
                employee.first_name,
                employee.last_name,
                employee.notes.as_ref().map(|s| s.as_str()),
                employee.sin,
                employee.address.street,
                employee.address.city,
                employee.address.province,
                employee.address.postal_code,
                employee.pay_type.as_str(),
                decimal_to_cents(&employee.pay_rate)?,
                employee.date_of_birth.format("%Y-%m-%d").to_string(),
                decimal_to_basis_points(&employee.vacation_pay_rate),
                decimal_to_cents_signed(&employee.vacation_balance)
                    .map_err(|e| DbError::InvalidData(format!(
                        "Employee #{} ({} {}): vacation_balance is invalid — {} (value: {})",
                        employee.employee_number, employee.first_name, employee.last_name,
                        e, employee.vacation_balance
                    )))?,
                decimal_to_cents_signed(&employee.vacation_balance_days)
                    .map_err(|e| DbError::InvalidData(format!(
                        "Employee #{} ({} {}): vacation_balance_days is invalid — {} (value: {})",
                        employee.employee_number, employee.first_name, employee.last_name,
                        e, employee.vacation_balance_days
                    )))?,
                decimal_to_basis_points(&employee.overtime_multiplier),
                employee.ei_exempt as i32,
                employee.cpp_exempt as i32,
                employee.hire_date.format("%Y-%m-%d").to_string(),
                employee.hire_province,
                employee.termination_date.map(|d| d.format("%Y-%m-%d").to_string()),
                employee.dental_benefit,
                employee.is_active as i32,
                id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Employee {} not found", id)));
        }

        Ok(())
    }

    pub fn delete(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM employee WHERE id = ?1", [id])?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Employee {} not found", id)));
        }

        Ok(())
    }

    pub fn list_active(&self) -> DbResult<Vec<Employee>> {
        self.list_with_filter("is_active = 1")
    }

    pub fn list_all(&self) -> DbResult<Vec<Employee>> {
        self.list_with_filter("1=1")
    }

    fn list_with_filter(&self, filter: &str) -> DbResult<Vec<Employee>> {
        let conn = self.conn.lock().unwrap();
        let query = format!(
            "SELECT id, employee_number, first_name, last_name, notes, sin,
                    address_street, address_city, address_province, address_postal_code,
                    pay_type, pay_rate,
                    date_of_birth, vacation_pay_rate, vacation_balance, vacation_balance_days, overtime_multiplier,
                    ei_exempt, cpp_exempt,
                    hire_date, hire_province, termination_date, dental_benefit, is_active, created_at
             FROM employee WHERE {} ORDER BY employee_number",
            filter
        );

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            // Read identity fields first for error context
            let employee_number: String = row.get(1)?;
            let first_name: String = row.get(2)?;
            let last_name: String = row.get(3)?;

            let pay_type_str: String = row.get(10)?;
            let pay_type = PayType::from_str(&pay_type_str)
                .ok_or_else(|| employee_data_error(&employee_number, &first_name, &last_name, "pay_type", format!("unknown type '{}' (expected Hourly, Weekly, Monthly, or Annual)", pay_type_str)))?;

            let pay_rate_cents: i64 = row.get(11)?;
            let pay_rate = cents_to_decimal(pay_rate_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "pay_rate", format!("{} (stored as {} cents)", e, pay_rate_cents)))?;

            let vacation_balance_cents: i64 = row.get(14)?;
            let vacation_balance = cents_to_decimal_signed(vacation_balance_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "vacation_balance", format!("{} (stored as {} cents)", e, vacation_balance_cents)))?;

            let vacation_balance_days_cents: i64 = row.get(15)?;
            let vacation_balance_days = cents_to_decimal_signed(vacation_balance_days_cents)
                .map_err(|e| employee_data_error(&employee_number, &first_name, &last_name, "vacation_balance_days", format!("{} (stored as {} cents)", e, vacation_balance_days_cents)))?;

            Ok(Employee {
                id: Some(row.get(0)?),
                employee_number,
                first_name,
                last_name,
                notes: row.get::<_, Option<String>>(4)?,
                sin: row.get(5)?,
                address: Address { street: row.get(6)?, city: row.get(7)?, province: row.get(8)?, postal_code: row.get(9)? },
                pay_type,
                pay_rate,
                date_of_birth: row.get(12)?,
                vacation_pay_rate: basis_points_to_decimal(row.get::<_, i64>(13)?),
                vacation_balance,
                vacation_balance_days,
                overtime_multiplier: basis_points_to_decimal(row.get::<_, i64>(16)?),
                ei_exempt: row.get::<_, i32>(17)? != 0,
                cpp_exempt: row.get::<_, i32>(18)? != 0,
                hire_date: row.get(19)?,
                hire_province: row.get(20)?,
                termination_date: row.get(21)?,
                dental_benefit: row.get::<_, i32>(22)?,
                is_active: row.get::<_, i32>(23)? != 0,
                created_at: row.get(24)?,
            })
        })?;

        let mut employees = Vec::new();
        for employee in rows {
            employees.push(employee?);
        }

        Ok(employees)
    }

    pub fn find_by_number(&self, employee_number: &str) -> DbResult<Option<Employee>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_number, first_name, last_name, notes, sin,
                    address_street, address_city, address_province, address_postal_code,
                    pay_type, pay_rate,
                    date_of_birth, vacation_pay_rate, vacation_balance, vacation_balance_days, overtime_multiplier,
                    ei_exempt, cpp_exempt,
                    hire_date, hire_province, termination_date, dental_benefit, is_active, created_at
             FROM employee WHERE employee_number = ?1",
        )?;

        let result = stmt.query_row([employee_number], |row| {
            // Read identity fields first for error context
            let emp_num: String = row.get(1)?;
            let first_name: String = row.get(2)?;
            let last_name: String = row.get(3)?;

            let pay_type_str: String = row.get(10)?;
            let pay_type = PayType::from_str(&pay_type_str)
                .ok_or_else(|| employee_data_error(&emp_num, &first_name, &last_name, "pay_type", format!("unknown type '{}' (expected Hourly, Weekly, Monthly, or Annual)", pay_type_str)))?;

            let pay_rate_cents: i64 = row.get(11)?;
            let pay_rate = cents_to_decimal(pay_rate_cents)
                .map_err(|e| employee_data_error(&emp_num, &first_name, &last_name, "pay_rate", format!("{} (stored as {} cents)", e, pay_rate_cents)))?;

            let vacation_balance_cents: i64 = row.get(14)?;
            let vacation_balance = cents_to_decimal_signed(vacation_balance_cents)
                .map_err(|e| employee_data_error(&emp_num, &first_name, &last_name, "vacation_balance", format!("{} (stored as {} cents)", e, vacation_balance_cents)))?;

            let vacation_balance_days_cents: i64 = row.get(15)?;
            let vacation_balance_days = cents_to_decimal_signed(vacation_balance_days_cents)
                .map_err(|e| employee_data_error(&emp_num, &first_name, &last_name, "vacation_balance_days", format!("{} (stored as {} cents)", e, vacation_balance_days_cents)))?;

            Ok(Employee {
                id: Some(row.get(0)?),
                employee_number: emp_num,
                first_name,
                last_name,
                notes: row.get::<_, Option<String>>(4)?,
                sin: row.get(5)?,
                address: Address { street: row.get(6)?, city: row.get(7)?, province: row.get(8)?, postal_code: row.get(9)? },
                pay_type,
                pay_rate,
                date_of_birth: row.get(12)?,
                vacation_pay_rate: basis_points_to_decimal(row.get::<_, i64>(13)?),
                vacation_balance,
                vacation_balance_days,
                overtime_multiplier: basis_points_to_decimal(row.get::<_, i64>(16)?),
                ei_exempt: row.get::<_, i32>(17)? != 0,
                cpp_exempt: row.get::<_, i32>(18)? != 0,
                hire_date: row.get(19)?,
                hire_province: row.get(20)?,
                termination_date: row.get(21)?,
                dental_benefit: row.get::<_, i32>(22)?,
                is_active: row.get::<_, i32>(23)? != 0,
                created_at: row.get(24)?,
            })
        });

        match result {
            Ok(employee) => Ok(Some(employee)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // =============================================
    // Employee Autofill CRUD Methods
    // =============================================

    /// Get all autofill values for an employee
    pub fn get_autofill(&self, employee_id: i64) -> DbResult<Vec<cpr_core::models::EmployeeAutofill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, autofill_type, type_name, amount, is_active, created_at
             FROM employee_autofill WHERE employee_id = ?1 ORDER BY autofill_type, type_name",
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            let autofill_type_str: String = row.get(2)?;
            let autofill_type = cpr_core::models::AutofillType::from_str(&autofill_type_str).ok_or(rusqlite::Error::InvalidQuery)?;

            Ok(cpr_core::models::EmployeeAutofill {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                autofill_type,
                type_name: row.get(3)?,
                amount: convert_cents(row.get::<_, i64>(4)?, "autofill_amount")?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        })?;

        let mut autofills = Vec::new();
        for row in rows {
            autofills.push(row?);
        }
        Ok(autofills)
    }

    /// Get active autofill values for an employee
    pub fn get_active_autofill(&self, employee_id: i64) -> DbResult<Vec<cpr_core::models::EmployeeAutofill>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, autofill_type, type_name, amount, is_active, created_at
             FROM employee_autofill WHERE employee_id = ?1 AND is_active = 1 ORDER BY autofill_type, type_name",
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            let autofill_type_str: String = row.get(2)?;
            let autofill_type = cpr_core::models::AutofillType::from_str(&autofill_type_str).ok_or(rusqlite::Error::InvalidQuery)?;

            Ok(cpr_core::models::EmployeeAutofill {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                autofill_type,
                type_name: row.get(3)?,
                amount: convert_cents(row.get::<_, i64>(4)?, "autofill_amount")?,
                is_active: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        })?;

        let mut autofills = Vec::new();
        for row in rows {
            autofills.push(row?);
        }
        Ok(autofills)
    }

    /// Save (insert or update) an autofill entry
    pub fn save_autofill(&self, autofill: &mut cpr_core::models::EmployeeAutofill) -> DbResult<i64> {
        autofill.validate().map_err(|e| DbError::InvalidData(e.to_string()))?;

        let conn = self.conn.lock().unwrap();

        if let Some(id) = autofill.id {
            // Update existing
            let rows_affected = conn.execute(
                "UPDATE employee_autofill SET
                    autofill_type = ?1, type_name = ?2, amount = ?3, is_active = ?4
                 WHERE id = ?5",
                params![autofill.autofill_type.as_str(), autofill.type_name, decimal_to_cents(&autofill.amount)?, autofill.is_active as i32, id,],
            )?;
            if rows_affected == 0 {
                return Err(DbError::NotFound("Autofill entry not found".to_string()));
            }
            Ok(id)
        } else {
            // Insert new - set created_at if not provided
            let created_at = autofill.created_at.unwrap_or_else(|| chrono::Utc::now());
            conn.execute(
                "INSERT INTO employee_autofill (employee_id, autofill_type, type_name, amount, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    autofill.employee_id,
                    autofill.autofill_type.as_str(),
                    autofill.type_name,
                    decimal_to_cents(&autofill.amount)?,
                    autofill.is_active as i32,
                    created_at.to_rfc3339(),
                ],
            )?;
            let id = conn.last_insert_rowid();
            autofill.id = Some(id);
            autofill.created_at = Some(created_at);
            Ok(id)
        }
    }

    /// Delete an autofill entry
    pub fn delete_autofill(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM employee_autofill WHERE id = ?1", params![id])?;
        if rows_affected == 0 {
            return Err(DbError::NotFound("Autofill entry not found".to_string()));
        }
        Ok(())
    }

    /// Delete all autofill entries for an employee
    pub fn delete_all_autofill(&self, employee_id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM employee_autofill WHERE employee_id = ?1", params![employee_id])?;
        Ok(())
    }
}
