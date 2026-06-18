use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, Row};
use rust_decimal::prelude::*;
use std::sync::{Arc, Mutex};

use crate::currency::{decimal_to_cents, read_numeric_as_i64, read_optional_hours_as_i64, try_cents};
use crate::{DbError, DbResult};
use cpr_core::models::payroll::YtdTotals;
use cpr_core::models::{Deductions, Payroll};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

/// Represents a distinct pay period in payroll history
/// Used for hierarchical navigation: year -> pay period -> payroll records
#[derive(Debug, Clone)]
pub struct PayrollPeriod {
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub pay_date: NaiveDate,
}

/// Filter options for listing payroll history
#[derive(Debug, Clone, Default)]
pub struct PayrollHistoryFilter {
    pub employee_id: Option<i64>,
    pub pay_date_from: Option<NaiveDate>,
    pub pay_date_to: Option<NaiveDate>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub struct PayrollHistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PayrollHistoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get a reference to the connection for direct queries (e.g., unlinking remittance)
    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn create(&self, payroll: &mut Payroll) -> DbResult<i64> {
        payroll.validate()?;

        self.create_import(payroll)
    }

    /// Create a history record without validation (for CSV import)
    /// This allows importing historical data with zero or negative values
    pub fn create_import(&self, payroll: &mut Payroll) -> DbResult<i64> {
        let gross_cents: i64 = crate::currency::decimal_to_cents(&payroll.gross_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_earnings_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.additional_earnings_total).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let insured_earning_cents: i64 = crate::currency::decimal_to_cents(&payroll.insured_earning).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_tax_amount_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.additional_tax_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp_cents: i64 = crate::currency::decimal_to_cents(&payroll.deductions.cpp).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp2_cents: i64 = crate::currency::decimal_to_cents(&payroll.deductions.cpp2).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let ei_cents: i64 = crate::currency::decimal_to_cents(&payroll.deductions.ei).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_tax_cents: i64 = crate::currency::decimal_to_cents(&payroll.deductions.federal_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_tax_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.deductions.provincial_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let additional_deductions_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.additional_deductions).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let net_pay_cents: i64 = crate::currency::decimal_to_cents(&payroll.net_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_personal_amount_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.federal_personal_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_personal_amount_cents: i64 =
            crate::currency::decimal_to_cents(&payroll.provincial_personal_amount).map_err(|e| DbError::InvalidData(e.to_string()))?;

        let regular_hours_thou: Option<i64> = payroll.regular_hours.map(|h| {
            let thou = (h * rust_decimal_macros::dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });
        let overtime_hours_thou: Option<i64> = payroll.overtime_hours.map(|h| {
            let thou = (h * rust_decimal_macros::dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO history (
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
                payroll.province,
                payroll.created_at.to_rfc3339(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        payroll.id = Some(id);

        // Insert additional deductions if any
        for deduction in &payroll.deductions.additional {
            let amount_cents = decimal_to_cents(&deduction.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            conn.execute(
                "INSERT INTO history_deduction (payroll_id, deduction_type, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, deduction.name, amount_cents, payroll.created_at.to_rfc3339(),],
            )?;
        }

        // Insert additional earnings if any
        for earning in &payroll.additional_earnings {
            let amount_cents = decimal_to_cents(&earning.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            let hours_thou: Option<i64> = earning.hours.map(|h| {
                let thou = (h * dec!(1000)).round_dp(0);
                thou.to_i64().unwrap_or(0)
            });
            conn.execute(
                "INSERT INTO history_earning (payroll_id, earning_type, amount, hours, is_periodic, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, earning.earning_type, amount_cents, hours_thou, earning.is_periodic as i32, payroll.created_at.to_rfc3339(),],
            )?;
        }

        Ok(id)
    }

    pub fn get(&self, id: i64) -> DbResult<Payroll> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount, cpp_deduction, cpp2_deduction, ei_deduction,
                    federal_tax, provincial_tax, additional_deductions, net_pay, federal_personal_amount,
                    provincial_personal_amount, province, created_at, remittance_id
             FROM history WHERE id = ?1",
        )?;

        let mut payroll = stmt.query_row([id], |row| {
            let pay_period_number: i64 = row.get(5)?;
            let total_pay_periods: i64 = row.get(6)?;
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let province: String = row.get(22)?;
            let created_at_str: String = row.get(23)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc);
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 11)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                gross_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 9)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_earnings_total: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 10)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_tax_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 12)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                deductions: Deductions {
                    cpp: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 13)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    cpp2: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 14)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    ei: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 15)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    federal_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 16)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    provincial_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 17)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    additional: vec![],
                },
                net_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 19)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                pay_period_number: if pay_period_number > 0 { Some(pay_period_number as i32) } else { None },
                total_pay_periods: total_pay_periods as i32,
                total_deductions: rust_decimal::Decimal::ZERO,
                additional_deductions: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 18)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                federal_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 20)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                provincial_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 21)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                province,
                remittance_id: row.get(24)?,
                created_at,
            })
        })?;

        // Load additional deductions
        let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;

        let deductions = deduction_stmt.query_map([id], |row| {
            Ok(cpr_core::models::AdditionalDeduction {
                name: row.get(0)?,
                amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })?;

        for deduction in deductions {
            payroll.deductions.additional.push(deduction?);
        }

        // Load additional earnings
        let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;

        let earnings = earning_stmt.query_map([id], |row| {
            let amount = crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
            let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let earning_type: String = row.get(0)?;
            let is_periodic = row.get::<_, i32>(3)? != 0;
            Ok(cpr_core::models::payroll::AdditionalEarning {
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

        Ok(payroll)
    }

    pub fn delete(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();

        // Delete additional deductions
        conn.execute("DELETE FROM history_deduction WHERE payroll_id = ?1", [id])?;

        // Delete additional earnings
        conn.execute("DELETE FROM history_earning WHERE payroll_id = ?1", [id])?;

        // Delete the main record
        let rows_affected = conn.execute("DELETE FROM history WHERE id = ?1", [id])?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Payroll {} not found", id)));
        }

        Ok(())
    }

    pub fn update(&self, payroll: &Payroll) -> DbResult<()> {
        payroll.validate()?;

        let id = payroll.id.ok_or_else(|| DbError::InvalidData("Payroll ID is required for update".to_string()))?;

        let regular_hours_thou: Option<i64> = payroll.regular_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });
        let overtime_hours_thou: Option<i64> = payroll.overtime_hours.map(|h| {
            let thou = (h * dec!(1000)).round_dp(0);
            thou.to_i64().unwrap_or(0)
        });

        let gross_cents = decimal_to_cents(&payroll.gross_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp_cents = decimal_to_cents(&payroll.deductions.cpp).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let cpp2_cents = decimal_to_cents(&payroll.deductions.cpp2).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let ei_cents = decimal_to_cents(&payroll.deductions.ei).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let federal_tax_cents = decimal_to_cents(&payroll.deductions.federal_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let provincial_tax_cents = decimal_to_cents(&payroll.deductions.provincial_tax).map_err(|e| DbError::InvalidData(e.to_string()))?;
        let net_pay_cents = decimal_to_cents(&payroll.net_pay).map_err(|e| DbError::InvalidData(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE history SET
                employee_id = ?1, pay_period_start_date = ?2, pay_period_end_date = ?3, pay_date = ?4,
                regular_hours = ?5, overtime_hours = ?6, gross_pay = ?7,
                cpp_deduction = ?8, cpp2_deduction = ?9, ei_deduction = ?10, federal_tax = ?11, provincial_tax = ?12,
                net_pay = ?13, province = ?14
             WHERE id = ?15",
            params![
                payroll.employee_id,
                payroll.pay_period_start.format("%Y-%m-%d").to_string(),
                payroll.pay_period_end.format("%Y-%m-%d").to_string(),
                payroll.pay_date.format("%Y-%m-%d").to_string(),
                regular_hours_thou,
                overtime_hours_thou,
                gross_cents,
                cpp_cents,
                cpp2_cents,
                ei_cents,
                federal_tax_cents,
                provincial_tax_cents,
                net_pay_cents,
                payroll.province,
                id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(DbError::NotFound(format!("Payroll {} not found", id)));
        }

        // Update additional deductions (delete and re-insert)
        conn.execute("DELETE FROM history_deduction WHERE payroll_id = ?1", [id])?;
        for deduction in &payroll.deductions.additional {
            let amount_cents = decimal_to_cents(&deduction.amount).map_err(|e| DbError::InvalidData(e.to_string()))?;
            conn.execute(
                "INSERT INTO history_deduction (payroll_id, deduction_type, amount, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, deduction.name, amount_cents, payroll.created_at.to_rfc3339(),],
            )?;
        }

        Ok(())
    }

    pub fn list_by_employee(&self, employee_id: i64) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount, cpp_deduction, cpp2_deduction, ei_deduction,
                    federal_tax, provincial_tax, additional_deductions, net_pay, federal_personal_amount,
                    provincial_personal_amount, province, created_at, remittance_id
             FROM history WHERE employee_id = ?1 ORDER BY pay_date DESC",
        )?;

        let rows = stmt.query_map([employee_id], |row| {
            let pay_period_number: i64 = row.get(5)?;
            let total_pay_periods: i64 = row.get(6)?;
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let province: String = row.get(22)?;
            let created_at_str: String = row.get(23)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc);
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 11)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                gross_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 9)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_earnings_total: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 10)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_tax_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 12)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                deductions: Deductions {
                    cpp: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 13)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    cpp2: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 14)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    ei: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 15)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    federal_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 16)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    provincial_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 17)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    additional: vec![],
                },
                net_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 19)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                pay_period_number: if pay_period_number > 0 { Some(pay_period_number as i32) } else { None },
                total_pay_periods: total_pay_periods as i32,
                total_deductions: Decimal::ZERO,
                additional_deductions: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 18)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                federal_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 20)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                provincial_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 21)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                province,
                remittance_id: row.get(24)?,
                created_at,
            })
        })?;

        let mut payrolls = Vec::new();
        for payroll_result in rows {
            let mut payroll = payroll_result?;

            // Load additional deductions
            let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;
            let deductions = deduction_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?;
            for deduction in deductions {
                payroll.deductions.additional.push(deduction?);
            }

            // Load additional earnings
            let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;
            let earnings = earning_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                let amount = crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
                let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
                let earning_type: String = row.get(0)?;
                let is_periodic = row.get::<_, i32>(3)? != 0;
                Ok(cpr_core::models::payroll::AdditionalEarning {
                    id: None,
                    payroll_id: payroll.id.unwrap_or(0),
                    earning_type,
                    amount,
                    hours,
                    is_periodic,
                })
            })?;
            for earning in earnings {
                payroll.additional_earnings.push(earning?);
            }

            payrolls.push(payroll);
        }

        Ok(payrolls)
    }

    pub fn get_ytd_totals(&self, employee_id: i64, year: i32) -> DbResult<YtdTotals> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT
                COALESCE(CAST(SUM(gross_pay) AS INTEGER), 0) as ytd_gross,
                COALESCE(CAST(SUM(cpp_deduction) AS INTEGER), 0) as ytd_cpp,
                COALESCE(CAST(SUM(cpp2_deduction) AS INTEGER), 0) as ytd_cpp2,
                COALESCE(CAST(SUM(ei_deduction) AS INTEGER), 0) as ytd_ei,
                COALESCE(CAST(SUM(federal_tax) AS INTEGER), 0) as ytd_federal,
                COALESCE(CAST(SUM(provincial_tax) AS INTEGER), 0) as ytd_provincial,
                COALESCE(CAST(SUM(net_pay) AS INTEGER), 0) as ytd_net
             FROM history
             WHERE employee_id = ?1
               AND strftime('%Y', pay_date) = ?2",
        )?;

        let totals = stmt.query_row(params![employee_id, year.to_string()], |row| {
            let gross_cents: i64 = row.get(0)?;
            let cpp_cents: i64 = row.get(1)?;
            let cpp2_cents: i64 = row.get(2)?;
            let ei_cents: i64 = row.get(3)?;
            let federal_cents: i64 = row.get(4)?;
            let provincial_cents: i64 = row.get(5)?;
            let net_cents: i64 = row.get(6)?;
            Ok(YtdTotals {
                employee_id,
                year,
                gross_pay: try_cents(gross_cents)?,
                cpp: try_cents(cpp_cents)?,
                cpp2: try_cents(cpp2_cents)?,
                ei: try_cents(ei_cents)?,
                federal_tax: try_cents(federal_cents)?,
                provincial_tax: try_cents(provincial_cents)?,
                net_pay: try_cents(net_cents)?,
                rpp_contributions: rust_decimal::Decimal::ZERO,
                pension_adjustment: rust_decimal::Decimal::ZERO,
            })
        })?;

        Ok(totals)
    }

    pub fn list_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount, cpp_deduction, cpp2_deduction, ei_deduction,
                    federal_tax, provincial_tax, additional_deductions, net_pay, federal_personal_amount,
                    provincial_personal_amount, province, created_at, remittance_id
             FROM history
             WHERE pay_date >= ?1 AND pay_date <= ?2
             ORDER BY pay_date DESC",
        )?;

        let rows = stmt.query_map(params![start.format("%Y-%m-%d").to_string(), end.format("%Y-%m-%d").to_string()], |row| {
            let pay_period_number: i64 = row.get(5)?;
            let total_pay_periods: i64 = row.get(6)?;
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let province: String = row.get(22)?;
            let created_at_str: String = row.get(23)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc);
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 11)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                gross_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 9)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_earnings_total: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 10)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_tax_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 12)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                deductions: Deductions {
                    cpp: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 13)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    cpp2: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 14)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    ei: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 15)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    federal_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 16)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    provincial_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 17)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    additional: vec![],
                },
                net_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 19)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                pay_period_number: if pay_period_number > 0 { Some(pay_period_number as i32) } else { None },
                total_pay_periods: total_pay_periods as i32,
                total_deductions: Decimal::ZERO,
                additional_deductions: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 18)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                federal_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 20)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                provincial_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 21)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                province,
                remittance_id: row.get(24)?,
                created_at,
            })
        })?;

        let mut payrolls = Vec::new();
        for payroll_result in rows {
            let mut payroll = payroll_result?;

            // Load additional deductions
            let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;
            let deductions = deduction_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?;
            for deduction in deductions {
                payroll.deductions.additional.push(deduction?);
            }

            // Load additional earnings
            let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;
            let earnings = earning_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                let amount = crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
                let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
                let earning_type: String = row.get(0)?;
                let is_periodic = row.get::<_, i32>(3)? != 0;
                Ok(cpr_core::models::payroll::AdditionalEarning {
                    id: None,
                    payroll_id: payroll.id.unwrap_or(0),
                    earning_type,
                    amount,
                    hours,
                    is_periodic,
                })
            })?;
            for earning in earnings {
                payroll.additional_earnings.push(earning?);
            }

            payrolls.push(payroll);
        }

        Ok(payrolls)
    }

    /// List payroll history with filters for search functionality
    pub fn list_filtered(&self, filter: &PayrollHistoryFilter) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();

        // Build dynamic query with optional filters
        let mut query = String::from(
            "SELECT h.id, h.employee_id, h.pay_period_start_date, h.pay_period_end_date, h.pay_date,
                    h.pay_period_number, h.total_pay_periods, h.regular_hours, h.overtime_hours, h.gross_pay,
                    h.additional_earnings, h.insured_earning, h.additional_tax_amount, h.cpp_deduction, h.cpp2_deduction, h.ei_deduction,
                    h.federal_tax, h.provincial_tax, h.additional_deductions, h.net_pay, h.federal_personal_amount,
                    h.provincial_personal_amount, h.province, h.created_at, h.remittance_id
             FROM history h
             LEFT JOIN employee e ON h.employee_id = e.id
             WHERE 1=1",
        );

        let mut param_values: Vec<String> = Vec::new();
        let mut param_index = 1;

        // Filter by employee_id
        if let Some(emp_id) = filter.employee_id {
            query.push_str(&format!(" AND h.employee_id = ?{}", param_index));
            param_values.push(emp_id.to_string());
            param_index += 1;
        }

        // Filter by pay_date_from
        if let Some(from_date) = &filter.pay_date_from {
            query.push_str(&format!(" AND h.pay_date >= ?{}", param_index));
            param_values.push(from_date.format("%Y-%m-%d").to_string());
            param_index += 1;
        }

        // Filter by pay_date_to
        if let Some(to_date) = &filter.pay_date_to {
            query.push_str(&format!(" AND h.pay_date <= ?{}", param_index));
            param_values.push(to_date.format("%Y-%m-%d").to_string());
            param_index += 1;
        }

        // Filter by search term (employee name or employee number)
        if let Some(search) = &filter.search_term {
            if !search.is_empty() {
                query.push_str(&format!(
                    " AND (e.first_name LIKE ?{} OR e.last_name LIKE ?{} OR e.employee_number LIKE ?{})",
                    param_index, param_index, param_index
                ));
                param_values.push(format!("%{}%", search));
                param_index += 1;
            }
        }

        // Order by pay_date descending
        query.push_str(" ORDER BY h.pay_date DESC");

        // Add limit and offset for pagination
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ?{}", param_index));
            param_values.push(limit.to_string());
            param_index += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ?{}", param_index));
            param_values.push(offset.to_string());
        }

        let mut stmt = conn.prepare(&query)?;

        // Convert param_values to rusqlite params
        let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| Self::row_to_payroll(row))?;

        let mut payrolls = Vec::new();
        for payroll_result in rows {
            let mut payroll = payroll_result?;

            // Load additional deductions
            let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;

            let deductions = deduction_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?;

            for deduction in deductions {
                payroll.deductions.additional.push(deduction?);
            }

            // Load additional earnings
            let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;

            let earnings = earning_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                let hours_thou: Option<i64> = row.get(2)?;
                let hours = hours_thou.map(|h| rust_decimal::Decimal::new(h, 3));
                let is_periodic: i32 = row.get(3)?;
                Ok(cpr_core::models::payroll::AdditionalEarning {
                    id: None,
                    payroll_id: payroll.id.unwrap_or(0),
                    earning_type: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    hours,
                    is_periodic: is_periodic != 0,
                })
            })?;

            for earning in earnings {
                payroll.additional_earnings.push(earning?);
            }

            payrolls.push(payroll);
        }

        Ok(payrolls)
    }

    /// Count total records matching filter (for pagination)
    pub fn count_filtered(&self, filter: &PayrollHistoryFilter) -> DbResult<i64> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from(
            "SELECT COUNT(*)
             FROM history h
             LEFT JOIN employee e ON h.employee_id = e.id
             WHERE 1=1",
        );

        let mut param_values: Vec<String> = Vec::new();
        let mut param_index = 1;

        if let Some(emp_id) = filter.employee_id {
            query.push_str(&format!(" AND h.employee_id = ?{}", param_index));
            param_values.push(emp_id.to_string());
            param_index += 1;
        }

        if let Some(from_date) = &filter.pay_date_from {
            query.push_str(&format!(" AND h.pay_date >= ?{}", param_index));
            param_values.push(from_date.format("%Y-%m-%d").to_string());
            param_index += 1;
        }

        if let Some(to_date) = &filter.pay_date_to {
            query.push_str(&format!(" AND h.pay_date <= ?{}", param_index));
            param_values.push(to_date.format("%Y-%m-%d").to_string());
            param_index += 1;
        }

        if let Some(search) = &filter.search_term {
            if !search.is_empty() {
                query.push_str(&format!(
                    " AND (e.first_name LIKE ?{} OR e.last_name LIKE ?{} OR e.employee_number LIKE ?{})",
                    param_index, param_index, param_index
                ));
                param_values.push(format!("%{}%", search));
            }
        }

        let mut stmt = conn.prepare(&query)?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();

        let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0))?;

        Ok(count)
    }

    /// List distinct years that have payroll history records
    /// Optionally filtered by employee_id
    pub fn list_years(&self, employee_id: Option<i64>) -> DbResult<Vec<i32>> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from(
            "SELECT DISTINCT strftime('%Y', pay_date) as year FROM history",
        );

        if employee_id.is_some() {
            query.push_str(" WHERE employee_id = ?1");
        }

        query.push_str(" ORDER BY year DESC");

        let mut stmt = conn.prepare(&query)?;

        let years: Vec<Result<i32, _>> = if let Some(emp_id) = employee_id {
            stmt.query_map([emp_id], |row| {
                let year_str: String = row.get(0)?;
                year_str.parse::<i32>().map_err(|_| rusqlite::Error::InvalidQuery)
            })?.collect()
        } else {
            stmt.query_map([], |row| {
                let year_str: String = row.get(0)?;
                year_str.parse::<i32>().map_err(|_| rusqlite::Error::InvalidQuery)
            })?.collect()
        };

        let mut result = Vec::new();
        for year in years {
            result.push(year?);
        }

        Ok(result)
    }

    /// List distinct pay periods for a given year
    /// Returns distinct combinations of pay_period_start_date, pay_period_end_date, and pay_date
    /// Optionally filtered by employee_id
    pub fn list_pay_periods(&self, year: i32, employee_id: Option<i64>) -> DbResult<Vec<PayrollPeriod>> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from(
            "SELECT DISTINCT pay_period_start_date, pay_period_end_date, pay_date FROM history WHERE strftime('%Y', pay_date) = ?1",
        );

        if employee_id.is_some() {
            query.push_str(" AND employee_id = ?2");
        }

        query.push_str(" ORDER BY pay_date DESC");


        let mut stmt = conn.prepare(&query)?;

        let periods: Vec<Result<PayrollPeriod, _>> = if let Some(emp_id) = employee_id {
            stmt.query_map(params![year.to_string(), emp_id], |row| {
                let start_str: String = row.get(0)?;
                let end_str: String = row.get(1)?;
                let pay_str: String = row.get(2)?;
                Ok(PayrollPeriod {
                    pay_period_start: NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                    pay_period_end: NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                    pay_date: NaiveDate::parse_from_str(&pay_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?.collect()
        } else {
            stmt.query_map([year.to_string()], |row| {
                let start_str: String = row.get(0)?;
                let end_str: String = row.get(1)?;
                let pay_str: String = row.get(2)?;
                Ok(PayrollPeriod {
                    pay_period_start: NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                    pay_period_end: NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                    pay_date: NaiveDate::parse_from_str(&pay_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?.collect()
        };

        let mut result = Vec::new();
        for period in periods {
            result.push(period?);
        }

        Ok(result)
    }

    /// Check if any history payroll records exist for the given dates
    /// Returns true if records exist with matching pay_period_start_date, pay_period_end_date, and pay_date
    pub fn exists_by_dates(&self, pay_period_start: NaiveDate, pay_period_end: NaiveDate, pay_date: NaiveDate) -> DbResult<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM history
             WHERE pay_period_start_date = ?1
               AND pay_period_end_date = ?2
               AND pay_date = ?3",
            params![pay_period_start.format("%Y-%m-%d").to_string(), pay_period_end.format("%Y-%m-%d").to_string(), pay_date.format("%Y-%m-%d").to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// List unremitted history payrolls (remittance_id IS NULL) within a date range
    /// Used by remittance summary/creation to prevent double-counting
    pub fn list_unremitted_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount, cpp_deduction, cpp2_deduction, ei_deduction,
                    federal_tax, provincial_tax, additional_deductions, net_pay, federal_personal_amount,
                    provincial_personal_amount, province, created_at, remittance_id
             FROM history
             WHERE pay_date >= ?1 AND pay_date <= ?2
               AND remittance_id IS NULL
             ORDER BY pay_date DESC",
        )?;

        let rows = stmt.query_map(params![start.format("%Y-%m-%d").to_string(), end.format("%Y-%m-%d").to_string()], |row| {
            let pay_period_number: i64 = row.get(5)?;
            let total_pay_periods: i64 = row.get(6)?;
            let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
            let province: String = row.get(22)?;
            let created_at_str: String = row.get(23)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc);
            Ok(Payroll {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                pay_period_start: row.get(2)?,
                pay_period_end: row.get(3)?,
                pay_date: row.get(4)?,
                regular_hours,
                overtime_hours,
                additional_earnings: vec![],
                insured_earning: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 11)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                gross_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 9)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_earnings_total: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 10)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional_tax_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 12)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                deductions: Deductions {
                    cpp: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 13)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    cpp2: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 14)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    ei: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 15)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    federal_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 16)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    provincial_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 17)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                    additional: vec![],
                },
                net_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 19)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                pay_period_number: if pay_period_number > 0 { Some(pay_period_number as i32) } else { None },
                total_pay_periods: total_pay_periods as i32,
                total_deductions: Decimal::ZERO,
                additional_deductions: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 18)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                federal_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 20)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                provincial_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 21)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                province,
                remittance_id: row.get(24)?,
                created_at,
            })
        })?;

        let mut payrolls = Vec::new();
        for payroll_result in rows {
            let mut payroll = payroll_result?;

            // Load additional deductions
            let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;
            let deductions = deduction_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?;
            for deduction in deductions {
                payroll.deductions.additional.push(deduction?);
            }

            // Load additional earnings
            let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;
            let earnings = earning_stmt.query_map([payroll.id.unwrap_or(0)], |row| {
                let amount = crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
                let hours = read_optional_hours_as_i64(row, 2)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
                let earning_type: String = row.get(0)?;
                let is_periodic = row.get::<_, i32>(3)? != 0;
                Ok(cpr_core::models::payroll::AdditionalEarning {
                    id: None,
                    payroll_id: payroll.id.unwrap_or(0),
                    earning_type,
                    amount,
                    hours,
                    is_periodic,
                })
            })?;
            for earning in earnings {
                payroll.additional_earnings.push(earning?);
            }

            payrolls.push(payroll);
        }

        Ok(payrolls)
    }

    /// Link unremitted history payrolls to a remittance record
    /// Updates remittance_id for all history records with pay_date <= cutoff and remittance_id IS NULL
    pub fn link_to_remittance(&self, remittance_id: i64, cutoff: NaiveDate) -> DbResult<usize> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE history SET remittance_id = ?1
             WHERE pay_date <= ?2 AND remittance_id IS NULL",
            params![remittance_id, cutoff.format("%Y-%m-%d").to_string()],
        )?;
        Ok(rows_affected)
    }

    /// Unlink all history payrolls from a remittance (used when deleting a remittance)
    pub fn unlink_from_remittance(&self, remittance_id: i64) -> DbResult<usize> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("UPDATE history SET remittance_id = NULL WHERE remittance_id = ?1", [remittance_id])?;
        Ok(rows_affected)
    }

    /// List history payrolls for a specific remittance
    pub fn list_by_remittance(&self, remittance_id: i64) -> DbResult<Vec<Payroll>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_period_start_date, pay_period_end_date, pay_date,
                    pay_period_number, total_pay_periods, regular_hours, overtime_hours, gross_pay,
                    additional_earnings, insured_earning, additional_tax_amount, cpp_deduction, cpp2_deduction, ei_deduction,
                    federal_tax, provincial_tax, additional_deductions, net_pay, federal_personal_amount,
                    provincial_personal_amount, province, created_at, remittance_id
             FROM history
             WHERE remittance_id = ?1
             ORDER BY pay_date ASC",
        )?;

        let rows = stmt.query_map([remittance_id], |row| Self::row_to_payroll(row))?;

        let mut payrolls = Vec::new();
        for payroll_result in rows {
            let mut payroll = payroll_result?;

            // Load additional deductions
            let mut deduction_stmt = conn.prepare("SELECT deduction_type, amount FROM history_deduction WHERE payroll_id = ?1")?;
            let deductions = deduction_stmt.query_map([payroll.id.unwrap()], |row| {
                Ok(cpr_core::models::AdditionalDeduction {
                    name: row.get(0)?,
                    amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                })
            })?;

            for deduction in deductions {
                payroll.deductions.additional.push(deduction?);
            }

            // Load additional earnings (for compatibility, even if not used in remittance report)
            let mut earning_stmt = conn.prepare("SELECT earning_type, amount, hours, is_periodic FROM history_earning WHERE payroll_id = ?1")?;
            let earnings = earning_stmt.query_map([payroll.id.unwrap()], |row| {
                let amount = crate::currency::cents_to_decimal(read_numeric_as_i64(row, 1)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
                let hours = row.get::<_, Option<i64>>(2)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
                let earning_type: String = row.get(0)?;
                let is_periodic = row.get::<_, i32>(3)? != 0;
                Ok(cpr_core::models::payroll::AdditionalEarning {
                    id: None,
                    payroll_id: payroll.id.unwrap(),
                    earning_type,
                    amount,
                    hours,
                    is_periodic,
                })
            })?;

            for earning in earnings {
                payroll.additional_earnings.push(earning?);
            }

            payrolls.push(payroll);
        }

        Ok(payrolls)
    }

    /// Helper to convert a database row to a Payroll struct
    fn row_to_payroll(row: &Row) -> rusqlite::Result<Payroll> {
        let pay_period_number: i64 = row.get(5)?;
        let total_pay_periods: i64 = row.get(6)?;
        let regular_hours = read_optional_hours_as_i64(row, 7)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
        let overtime_hours = read_optional_hours_as_i64(row, 8)?.map(|thou| rust_decimal::Decimal::new(thou, 3));
        let province: String = row.get(22)?;
        let created_at_str: String = row.get(23)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc);
        Ok(Payroll {
            id: Some(row.get(0)?),
            employee_id: row.get(1)?,
            pay_period_start: row.get(2)?,
            pay_period_end: row.get(3)?,
            pay_date: row.get(4)?,
            regular_hours,
            overtime_hours,
            additional_earnings: vec![],
            insured_earning: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 11)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            gross_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 9)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            additional_earnings_total: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 10)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            additional_tax_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 12)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            deductions: Deductions {
                cpp: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 13)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                cpp2: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 14)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                ei: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 15)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                federal_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 16)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                provincial_tax: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 17)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
                additional: vec![],
            },
            net_pay: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 19)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            pay_period_number: if pay_period_number > 0 { Some(pay_period_number as i32) } else { None },
            total_pay_periods: total_pay_periods as i32,
            total_deductions: Decimal::ZERO,
            additional_deductions: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 18)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            federal_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 20)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            provincial_personal_amount: crate::currency::cents_to_decimal(read_numeric_as_i64(row, 21)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
            province,
            remittance_id: row.get(24)?,
            created_at,
        })
    }
}
