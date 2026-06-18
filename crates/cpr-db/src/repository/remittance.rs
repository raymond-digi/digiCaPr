use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use chrono::{NaiveDate, DateTime, Utc};
use rust_decimal::Decimal;
use crate::{DbResult, DbError};
use crate::currency::{decimal_to_cents, read_numeric_as_i64, convert_cents};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Remittance {
    pub id: Option<i64>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_employees: i64,
    pub total_earnings: Decimal,
    pub total_cpp: Decimal,
    pub total_cpp2: Decimal,
    pub total_ei: Decimal,
    pub total_federal_tax: Decimal,
    pub total_provincial_tax: Decimal,
    pub grand_total: Decimal,
    pub cra_report_reference: Option<String>,
    pub generated_at: DateTime<Utc>,
}

pub struct RemittanceRepository {
    conn: Arc<Mutex<Connection>>,
}

impl RemittanceRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, remittance: &mut Remittance) -> DbResult<i64> {
        let conn = self.conn.lock().unwrap();
        let earnings_cents = decimal_to_cents(&remittance.total_earnings)?;
        let cpp_cents = decimal_to_cents(&remittance.total_cpp)?;
        let cpp2_cents = decimal_to_cents(&remittance.total_cpp2)?;
        let ei_cents = decimal_to_cents(&remittance.total_ei)?;
        let federal_cents = decimal_to_cents(&remittance.total_federal_tax)?;
        let provincial_cents = decimal_to_cents(&remittance.total_provincial_tax)?;
        let grand_cents = decimal_to_cents(&remittance.grand_total)?;

        conn.execute(
            "INSERT INTO remittance (period_start_date, period_end_date, total_employees, total_earnings, total_cpp, total_cpp2, total_ei, total_federal_tax, total_provincial_tax, grand_total, cra_report_reference, generated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                remittance.period_start.format("%Y-%m-%d").to_string(),
                remittance.period_end.format("%Y-%m-%d").to_string(),
                remittance.total_employees,
                earnings_cents,
                cpp_cents,
                cpp2_cents,
                ei_cents,
                federal_cents,
                provincial_cents,
                grand_cents,
                remittance.cra_report_reference.as_ref().map(|s| s.as_str()),
                remittance.generated_at.to_rfc3339()
            ],
        )?;
        let id = conn.last_insert_rowid();
        remittance.id = Some(id);
        Ok(id)
    }

    pub fn get(&self, id: i64) -> DbResult<Remittance> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, period_start_date, period_end_date, total_employees, total_earnings, total_cpp, total_cpp2, total_ei, total_federal_tax, total_provincial_tax, grand_total, cra_report_reference, generated_at FROM remittance WHERE id = ?1"
        )?;
        let remittance = stmt.query_row([id], |row| {
            Ok(Remittance {
                id: Some(row.get(0)?),
                period_start: row.get(1)?,
                period_end: row.get(2)?,
                total_employees: row.get(3)?,
                total_earnings: convert_cents(read_numeric_as_i64(row, 4)?, "total_earnings")?,
                total_cpp: convert_cents(read_numeric_as_i64(row, 5)?, "total_cpp")?,
                total_cpp2: convert_cents(read_numeric_as_i64(row, 6)?, "total_cpp2")?,
                total_ei: convert_cents(read_numeric_as_i64(row, 7)?, "total_ei")?,
                total_federal_tax: convert_cents(read_numeric_as_i64(row, 8)?, "total_federal_tax")?,
                total_provincial_tax: convert_cents(read_numeric_as_i64(row, 9)?, "total_provincial_tax")?,
                grand_total: convert_cents(read_numeric_as_i64(row, 10)?, "grand_total")?,
                cra_report_reference: row.get(11)?,
                generated_at: row.get(12)?,
            })
        })?;
        Ok(remittance)
    }

    pub fn list_by_period(&self, start: NaiveDate, end: NaiveDate) -> DbResult<Vec<Remittance>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, period_start_date, period_end_date, total_employees, total_earnings, total_cpp, total_cpp2, total_ei, total_federal_tax, total_provincial_tax, grand_total, cra_report_reference, generated_at FROM remittance WHERE period_start_date >= ?1 AND period_end_date <= ?2 ORDER BY period_start_date"
        )?;
        let rows = stmt.query_map(params![
            start.format("%Y-%m-%d").to_string(),
            end.format("%Y-%m-%d").to_string()
        ], |row| {
            Ok(Remittance {
                id: Some(row.get(0)?),
                period_start: row.get(1)?,
                period_end: row.get(2)?,
                total_employees: row.get(3)?,
                total_earnings: convert_cents(read_numeric_as_i64(row, 4)?, "total_earnings")?,
                total_cpp: convert_cents(read_numeric_as_i64(row, 5)?, "total_cpp")?,
                total_cpp2: convert_cents(read_numeric_as_i64(row, 6)?, "total_cpp2")?,
                total_ei: convert_cents(read_numeric_as_i64(row, 7)?, "total_ei")?,
                total_federal_tax: convert_cents(read_numeric_as_i64(row, 8)?, "total_federal_tax")?,
                total_provincial_tax: convert_cents(read_numeric_as_i64(row, 9)?, "total_provincial_tax")?,
                grand_total: convert_cents(read_numeric_as_i64(row, 10)?, "grand_total")?,
                cra_report_reference: row.get(11)?,
                generated_at: row.get(12)?,
            })
        })?;
        let mut remittances = Vec::new();
        for row_result in rows {
            remittances.push(row_result?);
        }
        Ok(remittances)
    }

    pub fn delete(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM remittance WHERE id = ?1", [id])?;
        if rows_affected == 0 {
            return Err(DbError::NotFound("Remittance not found".to_string()));
        }
        Ok(())
    }

    /// Get distinct years from all remittances, sorted DESC
    pub fn get_available_years(&self) -> DbResult<Vec<i32>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT CAST(strftime('%Y', period_start_date) AS INTEGER)
             FROM remittance
             ORDER BY 1 DESC"
        )?;
        let years: Vec<i32> = stmt.query_map([], |row| row.get(0))?.filter_map(Result::ok).collect();
        Ok(years)
    }

    /// List remittances for a specific year (Jan 1 to Dec 31)
    pub fn list_by_year(&self, year: i32) -> DbResult<Vec<Remittance>> {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).ok_or_else(|| DbError::InvalidData("Invalid year".to_string()))?;
        let end = NaiveDate::from_ymd_opt(year, 12, 31).ok_or_else(|| DbError::InvalidData("Invalid year".to_string()))?;
        self.list_by_period(start, end)
    }
}