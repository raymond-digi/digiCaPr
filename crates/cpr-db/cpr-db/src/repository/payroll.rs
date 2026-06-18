use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use rust_decimal::prelude::*;
use chrono::{NaiveDate};

use rust_decimal::prelude::ToPrimitive;
use crate::currency::{decimal_to_cents, convert_cents, read_numeric_as_i64, read_optional_hours_as_i64};
use rust_decimal_macros::dec;
use cpr_core::models::payroll::{Payroll, Deductions, AdditionalEarning};
use crate::{DbResult, DbError};

pub struct PayrollRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PayrollRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get employee's province from employee table
    fn get_employee_province(&self, employee_id: i64) -> DbResult<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT address_province FROM employee WHERE id = ?1"
        )?;
        
        let province: String = stmt.query_row([employee_id], |row| {
            row.get(0)
        })?;
        Ok(province)
    }

    pub fn create(&self, payroll: &mut Payroll) -> DbResult<i64> {
        payroll.validate()?;

        // Get employee's province from employee table
        let employee_province = self.get_employee_province(payroll.employee_id)?;
        
        // Personal amounts should already be set by the command layer during calculation
        // Just ensure province is set for consistency
        if payroll.province.is_empty() {
            payroll.province = employee_province.clone();
        }

        let gross_cents: i64 = decimal_to_cents(&payroll.gross_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_earnings_cents: i64 = decimal_to_cents(&payroll.additional_earnings_total).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let insured_earning_cents: i64 = decimal_to_cents(&payroll.insured_earning).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_tax_amount_cents: i64 = decimal_to_cents(&payroll.additional_tax_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp_cents: i64 = decimal_to_cents(&payroll.deductions.cpp).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp2_cents: i64 = decimal_to_cents(&payroll.deductions.cpp2).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let ei_cents: i64 = decimal_to_cents(&payroll.deductions.ei).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_tax_cents: i64 = decimal_to_cents(&payroll.deductions.federal_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_tax_cents: i64 = decimal_to_cents(&payroll.deductions.provincial_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_deductions_cents: i64 = decimal_to_cents(&payroll.additional_deductions).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let net_pay_cents: i64 = decimal_to_cents(&payroll.net_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_personal_amount_cents: i64 = decimal_to_cents(&payroll.federal_personal_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_personal_amount_cents: i64 = decimal_to_cents(&payroll.provincial_personal_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;

        let regular_hours_thou: Option<i64> = payroll.regular_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });
        let overtime_hours_thou: Option<i64> = payroll.overtime_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO payroll (
                employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                additional_earnings, insured_earning, additional_tax_amount,
                cpp_deduction, cpp2_deduction, ei_deduction, federal_tax, provincial_tax,
                additional_deductions, net_pay, federal_personal_amount, provincial_personal_amount, province, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
            params![
                payroll.employee_id,
                payroll.pay_period_start.format("%Y-%m-%d").to_string(),
                payroll.pay_period_end.format("%Y-%m-%d").to_string(),
                payroll.pay_date.format("%Y-%m-%d").to_string(),
                payroll.pay_period_number.unwrap_or(0),
                payroll.total_pay_periods,
                regular_hours_thou,
                overtime_hours_thou,
                gross_cents,
                additional_earnings_cents,
                insured_earning_cents,
                additional_tax_amount_cents,
                cpp_cents,
                cpp2_cents,
                ei_cents,
                federal_tax_cents,
                provincial_tax_cents,
                additional_deductions_cents,
                net_pay_cents,
                federal_personal_amount_cents,
                provincial_personal_amount_cents,
                &payroll.province,
                payroll.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        payroll.id = Some(id);

        // Insert additional deductions if any
        for deduction in &payroll.deductions.additional {
            let amount_cents = decimal_to_cents(&deduction.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            // Convert deduction_type to lowercase for consistency
            let deduction_type_lower = deduction.name.to_lowercase();
            conn.execute(
                "INSERT INTO payroll_deduction (payroll_id, deduction_type, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    id,
                    deduction_type_lower,
                    amount_cents,
                    payroll.created_at.to_rfc3339(),
                ],
            )?;
        }

        // Insert additional earnings if any
        for earning in &payroll.additional_earnings {
            let amount_cents = decimal_to_cents(&earning.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            let hours_thou: Option<i64> = earning.hours.map(|h| {
                let thou = (h * dec!(1000)).round_dp(0);
                thou.to_i64().unwrap_or(0)
            });
            // Convert earning_type to lowercase for consistency
            let earning_type_lower = earning.earning_type.to_lowercase();
            conn.execute(
                "INSERT INTO payroll_earning (payroll_id, earning_type, amount, hours, is_periodic, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    earning_type_lower,
                    amount_cents,
                    hours_thou,
                    earning.is_periodic as i32,
                    payroll.created_at.to_rfc3339(),
                ],
            )?;
        }

        Ok(id)
    }

    pub fn get(&self, id: i64) -> DbResult<Payroll> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount,
                    cpp_deduction, cpp2_deduction, ei_deduction, federal_tax, provincial_tax,
                    additional_deductions, net_pay, federal_personal_amount, provincial_personal_amount, province, created_at
             FROM payroll WHERE id = ?1"
        )?;

        let mut payroll = stmt.query_row([id], |row| {
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| Decimal::new(thou, 3));
            let pay_period_number = row.get::<_, Option<i32>>(5)?;
            let total_pay_periods = row.get::<_, i32>(6)?;
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                pay_period_number,
                total_pay_periods,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: convert_cents(read_numeric_as_i64(row, 11)?, "insured_earning")?,
                gross_pay: convert_cents(read_numeric_as_i64(row, 9)?, "gross_pay")?,
                additional_earnings_total: convert_cents(read_numeric_as_i64(row, 10)?, "additional_earnings_total")?,
                additional_tax_amount: convert_cents(read_numeric_as_i64(row, 12)?, "additional_tax_amount")?,
                deductions: Deductions {
                    cpp: convert_cents(read_numeric_as_i64(row, 13)?, "cpp")?,
                    cpp2: convert_cents(read_numeric_as_i64(row, 14)?, "cpp2")?,
                    ei: convert_cents(read_numeric_as_i64(row, 15)?, "ei")?,
                    federal_tax: convert_cents(read_numeric_as_i64(row, 16)?, "federal_tax")?,
                    provincial_tax: convert_cents(read_numeric_as_i64(row, 17)?, "provincial_tax")?,
                    additional: vec![],
                },
                net_pay: convert_cents(read_numeric_as_i64(row, 19)?, "net_pay")?,
                total_deductions: Decimal::ZERO,
                additional_deductions: convert_cents(read_numeric_as_i64(row, 18)?, "additional_deductions")?,
                federal_personal_amount: convert_cents(read_numeric_as_i64(row, 20)?, "federal_personal_amount")?,
                provincial_personal_amount: convert_cents(read_numeric_as_i64(row, 21)?, "provincial_personal_amount")?,
                province: row.get(22)?,
                remittance_id: None,
                created_at: row.get(23)?,
            })
        })?;

        // Load additional deductions
        let mut deduction_stmt = conn.prepare(
            "SELECT deduction_type, amount FROM payroll_deduction WHERE payroll_id = ?1 ORDER BY id"
        )?;

        let deductions = deduction_stmt.query_map([id], |row| {
            Ok(cpr_core::models::AdditionalDeduction {
                name: row.get(0)?,
                amount: convert_cents(read_numeric_as_i64(row, 1)?, "deduction_amount")?,
            })
        })?;

        for deduction in deductions {
            payroll.deductions.additional.push(deduction?);
        }

        // Load additional earnings
        let mut earning_stmt = conn.prepare(
            "SELECT earning_type, amount, hours, is_periodic FROM payroll_earning WHERE payroll_id = ?1 ORDER BY id"
        )?;

        let earnings = earning_stmt.query_map([id], |row| {
            let amount = convert_cents(read_numeric_as_i64(row, 1)?, "earning_amount")?;
            let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| Decimal::new(thou, 3));
            let earning_type: String = row.get(0)?;
            let is_periodic = row.get::<_, i32>(3)? != 0;
            Ok(AdditionalEarning {
                id: None,
                payroll_id: id,
                earning_type,
                amount,
                hours,
                is_periodic,
            })
        })?;

        for earning in earnings {
            payroll.additional_earnings.push(earning?);
        }

        payroll.calculate_net_pay();

        Ok(payroll)
    }

    pub fn update(&self, payroll: &Payroll) -> DbResult<()> {
        payroll.validate()?;

        let id = payroll.id.ok_or_else(||
            DbError::InvalidData("Payroll ID is required for update".to_string())
        )?;

        let gross_cents: i64 = decimal_to_cents(&payroll.gross_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_earnings_cents: i64 = decimal_to_cents(&payroll.additional_earnings_total).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let insured_earning_cents: i64 = decimal_to_cents(&payroll.insured_earning).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_tax_amount_cents: i64 = decimal_to_cents(&payroll.additional_tax_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp_cents: i64 = decimal_to_cents(&payroll.deductions.cpp).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp2_cents: i64 = decimal_to_cents(&payroll.deductions.cpp2).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let ei_cents: i64 = decimal_to_cents(&payroll.deductions.ei).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_tax_cents: i64 = decimal_to_cents(&payroll.deductions.federal_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_tax_cents: i64 = decimal_to_cents(&payroll.deductions.provincial_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_deductions_cents: i64 = decimal_to_cents(&payroll.additional_deductions).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let net_pay_cents: i64 = decimal_to_cents(&payroll.net_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;

        let regular_hours_thou: Option<i64> = payroll.regular_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });
        let overtime_hours_thou: Option<i64> = payroll.overtime_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });

        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE payroll SET
                employee_id = ?1, pay_period_start_date = ?2, pay_period_end_date = ?3, pay_date = ?4,
                pay_period_number = ?5, total_pay_periods = ?6, regular_hours = ?7, overtime_hours = ?8, gross_pay = ?9,
                additional_earnings = ?10, insured_earning = ?11, additional_tax_amount = ?12,
                cpp_deduction = ?13, cpp2_deduction = ?14, ei_deduction = ?15, federal_tax = ?16, provincial_tax = ?17,
                additional_deductions = ?18, net_pay = ?19
             WHERE id = ?20",
            params![
                payroll.employee_id,
                payroll.pay_period_start.format("%Y-%m-%d").to_string(),
                payroll.pay_period_end.format("%Y-%m-%d").to_string(),
                payroll.pay_date.format("%Y-%m-%d").to_string(),
                payroll.pay_period_number.unwrap_or(0),
                payroll.total_pay_periods,
                regular_hours_thou,
                overtime_hours_thou,
                gross_cents,
                additional_earnings_cents,
                insured_earning_cents,
                additional_tax_amount_cents,
                cpp_cents,
                cpp2_cents,
                ei_cents,
                federal_tax_cents,
                provincial_tax_cents,
                additional_deductions_cents,
                net_pay_cents,
                id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Payroll {} not found", id)));
        }

        // Delete related records first
        conn.execute("DELETE FROM payroll_earning WHERE payroll_id = ?1", params![id])?;
        conn.execute("DELETE FROM payroll_deduction WHERE payroll_id = ?1", params![id])?;

        // Insert additional deductions if any
        for deduction in &payroll.deductions.additional {
            let amount_cents = decimal_to_cents(&deduction.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            // Convert deduction_type to lowercase for consistency
            let deduction_type_lower = deduction.name.to_lowercase();
            conn.execute(
                "INSERT INTO payroll_deduction (payroll_id, deduction_type, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    id,
                    deduction_type_lower,
                    amount_cents,
                    payroll.created_at.to_rfc3339(),
                ],
            )?;
        }

        // Insert additional earnings if any
        for earning in &payroll.additional_earnings {
            let amount_cents = decimal_to_cents(&earning.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            let hours_thou: Option<i64> = earning.hours.map(|h| {
                let thou = (h * dec!(1000)).round_dp(0);
                thou.to_i64().unwrap_or(0)
            });
            // Convert earning_type to lowercase for consistency
            let earning_type_lower = earning.earning_type.to_lowercase();
            conn.execute(
                "INSERT INTO payroll_earning (payroll_id, earning_type, amount, hours, is_periodic, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    earning_type_lower,
                    amount_cents,
                    hours_thou,
                    earning.is_periodic as i32,
                    payroll.created_at.to_rfc3339(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn list_by_employee(&self, employee_id: i64) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    regular_hours, overtime_hours, gross_pay,
                    cpp_deduction, cpp2_deduction, ei_deduction, federal_tax, provincial_tax,
                    net_pay, federal_personal_amount, provincial_personal_amount, province, created_at
             FROM payroll WHERE employee_id = ?1 ORDER BY pay_date DESC"
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                regular_hours: read_optional_hours_as_i64(row, 5)?.map(|thou| Decimal::new(thou, 3)),
                overtime_hours: read_optional_hours_as_i64(row, 6)?.map(|thou| Decimal::new(thou, 3)),
                additional_earnings: vec![],
                insured_earning: Decimal::ZERO,
                gross_pay: convert_cents(row.get::<_, i64>(7)?, "gross_pay")?,
                additional_earnings_total: Decimal::ZERO,
                additional_tax_amount: Decimal::ZERO,
                deductions: Deductions {
                    cpp: convert_cents(row.get::<_, i64>(8)?, "cpp")?,
                    cpp2: convert_cents(row.get::<_, i64>(9)?, "cpp2")?,
                    ei: convert_cents(row.get::<_, i64>(10)?, "ei")?,
                    federal_tax: convert_cents(row.get::<_, i64>(11)?, "federal_tax")?,
                    provincial_tax: convert_cents(row.get::<_, i64>(12)?, "provincial_tax")?,
                    additional: vec![],
                },
                net_pay: convert_cents(row.get::<_, i64>(13)?, "net_pay")?,
                pay_period_number: None,
                total_pay_periods: 0,
                total_deductions: Decimal::ZERO,
                additional_deductions: Decimal::ZERO,
                federal_personal_amount: convert_cents(row.get::<_, i64>(14)?, "federal_personal_amount")?,
                provincial_personal_amount: convert_cents(row.get::<_, i64>(15)?, "provincial_personal_amount")?,
                province: row.get(16)?,
                remittance_id: None,
                created_at: row.get(17)?,
            })
        })?;

        let mut payrolls = Vec::new();
        for payroll in rows {
            payrolls.push(payroll?);
        }

        Ok(payrolls)
    }

    pub fn list_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    regular_hours, overtime_hours, gross_pay,
                    cpp_deduction, cpp2_deduction, ei_deduction, federal_tax, provincial_tax,
                    net_pay, federal_personal_amount, provincial_personal_amount, province, created_at
             FROM payroll
             WHERE pay_date >= ?1 AND pay_date <= ?2
             ORDER BY pay_date DESC"
        )?;

        let rows = stmt.query_map(
            params![
                start.format("%Y-%m-%d").to_string(),
                end.format("%Y-%m-%d").to_string()
            ],
            |row| {
                Ok(Payroll {
                    id: Some(row.get(0)?),
                    employee_id: row.get(1)?,
                    pay_period_start: row.get(2)?,
                    pay_period_end: row.get(3)?,
                    pay_date: row.get(4)?,
                    regular_hours: read_optional_hours_as_i64(row, 5)?.map(|thou| Decimal::new(thou, 3)),
                    overtime_hours: read_optional_hours_as_i64(row, 6)?.map(|thou| Decimal::new(thou, 3)),
                    additional_earnings: vec![],
                    insured_earning: Decimal::ZERO,
                    gross_pay: convert_cents(row.get::<_, i64>(7)?, "gross_pay")?,
                    additional_earnings_total: Decimal::ZERO,
                    additional_tax_amount: Decimal::ZERO,
                    deductions: Deductions {
                        cpp: convert_cents(row.get::<_, i64>(8)?, "cpp")?,
                        cpp2: convert_cents(row.get::<_, i64>(9)?, "cpp2")?,
                        ei: convert_cents(row.get::<_, i64>(10)?, "ei")?,
                        federal_tax: convert_cents(row.get::<_, i64>(11)?, "federal_tax")?,
                        provincial_tax: convert_cents(row.get::<_, i64>(12)?, "provincial_tax")?,
                        additional: vec![],
                    },
                    net_pay: convert_cents(row.get::<_, i64>(13)?, "net_pay")?,
                    pay_period_number: None,
                    total_pay_periods: 0,
                    total_deductions: Decimal::ZERO,
                    additional_deductions: Decimal::ZERO,
                    federal_personal_amount: convert_cents(row.get::<_, i64>(14)?, "federal_personal_amount")?,
                    provincial_personal_amount: convert_cents(row.get::<_, i64>(15)?, "provincial_personal_amount")?,
                    province: row.get(16)?,
                    remittance_id: None,
                    created_at: row.get(17)?,
                })
            }
        )?;

        let mut payrolls = Vec::new();
        for payroll in rows {
            payrolls.push(payroll?);
        }

        Ok(payrolls)
    }

    pub fn delete(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        
        // Delete related records first
        conn.execute("DELETE FROM payroll_earning WHERE payroll_id = ?1", params![id])?;
        conn.execute("DELETE FROM payroll_deduction WHERE payroll_id = ?1", params![id])?;
        
        // Delete main record
        let rows_affected = conn.execute("DELETE FROM payroll WHERE id = ?1", params![id])?;
        
        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Current payroll {} not found", id)));
        }
        
        Ok(())
    }

    pub fn get_dates(&self) -> DbResult<Option<(NaiveDate, NaiveDate, NaiveDate)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT pay_period_start_date, pay_period_end_date, pay_date FROM payroll ORDER BY pay_date DESC LIMIT 1"
        )?;
        match stmt.query_row([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        }) {
            Ok(dates) => Ok(Some(dates)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub fn list_all(&self) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount,
                    cpp_deduction, cpp2_deduction, ei_deduction, federal_tax, provincial_tax,
                    additional_deductions, net_pay, federal_personal_amount, provincial_personal_amount, province, created_at
             FROM payroll ORDER BY pay_date DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| Decimal::new(thou, 3));
            let pay_period_number = row.get::<_, Option<i32>>(5)?;
            let total_pay_periods = row.get::<_, i32>(6)?;
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                pay_period_number,
                total_pay_periods,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: convert_cents(read_numeric_as_i64(row, 11)?, "insured_earning")?,
                gross_pay: convert_cents(read_numeric_as_i64(row, 9)?, "gross_pay")?,
                additional_earnings_total: convert_cents(read_numeric_as_i64(row, 10)?, "additional_earnings_total")?,
                additional_tax_amount: convert_cents(read_numeric_as_i64(row, 12)?, "additional_tax_amount")?,
                deductions: Deductions {
                    cpp: convert_cents(read_numeric_as_i64(row, 13)?, "cpp")?,
                    cpp2: convert_cents(read_numeric_as_i64(row, 14)?, "cpp2")?,
                    ei: convert_cents(read_numeric_as_i64(row, 15)?, "ei")?,
                    federal_tax: convert_cents(read_numeric_as_i64(row, 16)?, "federal_tax")?,
                    provincial_tax: convert_cents(read_numeric_as_i64(row, 17)?, "provincial_tax")?,
                    additional: vec![],
                },
                net_pay: convert_cents(read_numeric_as_i64(row, 19)?, "net_pay")?,
                total_deductions: Decimal::ZERO,
                additional_deductions: convert_cents(read_numeric_as_i64(row, 18)?, "additional_deductions")?,
                federal_personal_amount: convert_cents(read_numeric_as_i64(row, 20)?, "federal_personal_amount")?,
                provincial_personal_amount: convert_cents(read_numeric_as_i64(row, 21)?, "provincial_personal_amount")?,
                province: row.get(22)?,
                remittance_id: None,
                created_at: row.get(23)?,
            })
        })?;

        // Collect all payroll IDs first to avoid borrowing issues
        let mut payrolls: Vec<Payroll> = Vec::new();
        for payroll_result in rows {
            payrolls.push(payroll_result?);
        }
        
        // Now load additional data for each payroll
        for payroll in &mut payrolls {
            let payroll_id = payroll.id.unwrap();
            
            // Load additional deductions
            let mut deduction_stmt = conn.prepare(
                "SELECT deduction_type, amount FROM payroll_deduction WHERE payroll_id = ?1 ORDER BY id"
            )?;
            let deduction_rows = deduction_stmt.query_map([payroll_id], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: convert_cents(row.get::<_, i64>(1)?, "deduction_amount")?,
                })
            })?;
            for deduction_result in deduction_rows {
                payroll.deductions.additional.push(deduction_result?);
            }
            
            // Load additional earnings
            let mut earning_stmt = conn.prepare(
                "SELECT earning_type, amount, hours, is_periodic FROM payroll_earning WHERE payroll_id = ?1 ORDER BY id"
            )?;
            let earning_rows = earning_stmt.query_map([payroll_id], |row| {
                let amount = convert_cents(row.get::<_, i64>(1)?, "earning_amount")?;
                let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| Decimal::new(thou, 3));
                let earning_type: String = row.get(0)?;
                let is_periodic = row.get::<_, i32>(3)? != 0;
                Ok(AdditionalEarning {
                    id: None,
                    payroll_id,
                    earning_type,
                    amount,
                    hours,
                    is_periodic,
                })
            })?;
            for earning_result in earning_rows {
                payroll.additional_earnings.push(earning_result?);
            }
            
            // Recalculate totals
            payroll.calculate_net_pay();
        }
        
        Ok(payrolls)
    }

    /// Clear all current payroll records and related tables
    pub fn clear_all(&self) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        // Delete child tables first
        conn.execute("DELETE FROM payroll_earning", [])?;
        conn.execute("DELETE FROM payroll_deduction", [])?;
        conn.execute("DELETE FROM payroll", [])?;
        Ok(())
    }
}