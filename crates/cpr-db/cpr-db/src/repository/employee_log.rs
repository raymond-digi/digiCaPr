use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, NaiveDate, Utc};

use cpr_core::models::{PayRateHistory, EmploymentHistory, PayType};
use crate::currency::{decimal_to_cents};
use crate::{DbError, DbResult};

pub struct EmployeeHistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl EmployeeHistoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    // ========== Pay Rate History ==========
    
    /// Add a new pay rate history entry
    pub fn add_pay_rate(&self, history: &mut PayRateHistory) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO pay_rate_log
             (employee_id, pay_rate, pay_type, effective_date, end_date, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                history.employee_id,
                decimal_to_cents(&history.pay_rate)?,
                history.pay_type.as_str(),
                history.effective_date.format("%Y-%m-%d").to_string(),
                history.end_date.map(|d| d.format("%Y-%m-%d").to_string()),
                history.reason.as_deref(),
                history.created_at.to_rfc3339(),
            ],
        )?;
        
        history.id = Some(conn.last_insert_rowid());
        Ok(())
    }
    
    /// Get all pay rate history for an employee
    pub fn get_pay_rate_history(&self, employee_id: i64) -> DbResult<Vec<PayRateHistory>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_rate, pay_type, effective_date, end_date, reason, created_at
             FROM pay_rate_log
             WHERE employee_id = ?1
             ORDER BY effective_date DESC"
        )?;
        
        let histories = stmt.query_map([employee_id], |row| {
            Self::row_to_pay_rate_history(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(histories)
    }
    
    /// Get current pay rate for an employee
    pub fn get_current_pay_rate(&self, employee_id: i64) -> DbResult<Option<PayRateHistory>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, pay_rate, pay_type, effective_date, end_date, reason, created_at
             FROM pay_rate_log
             WHERE employee_id = ?1 AND end_date IS NULL
             ORDER BY effective_date DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query([employee_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_pay_rate_history(row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Close all open pay rate history entries on a specific date
    pub fn close_pay_rate_history(&self, employee_id: i64, end_date: NaiveDate) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE pay_rate_log
             SET end_date = ?1
             WHERE employee_id = ?2 AND end_date IS NULL",
            params![end_date.to_string(), employee_id],
        )?;
        
        Ok(())
    }
    
    /// Delete a pay rate history entry
    pub fn delete_pay_rate(&self, id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "DELETE FROM pay_rate_log WHERE id = ?1",
            [id],
        )?;
        
        Ok(())
    }
    
    fn row_to_pay_rate_history(row: &Row) -> rusqlite::Result<PayRateHistory> {
        Ok(PayRateHistory {
            id: Some(row.get(0)?),
            employee_id: row.get(1)?,
            pay_rate: {
                let cents: i64 = row.get(2)?;
                crate::currency::convert_cents(cents, "pay_rate")?
            },
            pay_type: {
                let type_str: String = row.get(3)?;
                PayType::from_str(&type_str).ok_or(rusqlite::Error::InvalidQuery)?
            },
            effective_date: {
                let date_str: String = row.get(4)?;
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?
            },
            end_date: {
                let date_opt: Option<String> = row.get(5)?;
                date_opt.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            },
            reason: row.get::<_, Option<String>>(6)?,
            created_at: {
                let dt_str: String = row.get(7)?;
                DateTime::parse_from_rfc3339(&dt_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc)
            },
        })
    }
    
    // ========== Employment History ==========
    
    /// Add a new employment history entry
    pub fn add_employment(&self, history: &mut EmploymentHistory) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO employment_log
             (employee_id, hire_date, termination_date, termination_reason, rehire_eligible, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                history.employee_id,
                history.hire_date.to_string(),
                history.termination_date.map(|d| d.to_string()),
                history.termination_reason,
                if history.rehire_eligible { 1 } else { 0 },
                history.notes,
                history.created_at.to_rfc3339(),
            ],
        )?;
        
        history.id = Some(conn.last_insert_rowid());
        Ok(())
    }
    
    /// Get all employment history for an employee
    pub fn get_employment_history(&self, employee_id: i64) -> DbResult<Vec<EmploymentHistory>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, hire_date, termination_date, termination_reason,
                    rehire_eligible, notes, created_at
             FROM employment_log
             WHERE employee_id = ?1
             ORDER BY hire_date DESC"
        )?;
        
        let histories = stmt.query_map([employee_id], |row| {
            Self::row_to_employment_history(row)
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(histories)
    }
    
    /// Get current employment period
    pub fn get_current_employment(&self, employee_id: i64) -> DbResult<Option<EmploymentHistory>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, hire_date, termination_date, termination_reason,
                    rehire_eligible, notes, created_at
             FROM employment_log
             WHERE employee_id = ?1 AND termination_date IS NULL
             ORDER BY hire_date DESC
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query([employee_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_employment_history(row)?))
        } else {
            Ok(None)
        }
    }
    
    /// Update employment history (for termination)
    pub fn update_employment(&self, history: &EmploymentHistory) -> DbResult<()> {
        let id = history.id.ok_or_else(|| DbError::NotFound("Employment history ID not found".to_string()))?;
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE employment_log
             SET termination_date = ?1,
                 termination_reason = ?2,
                 rehire_eligible = ?3,
                 notes = ?4
             WHERE id = ?5",
            params![
                history.termination_date.map(|d| d.to_string()),
                history.termination_reason,
                if history.rehire_eligible { 1 } else { 0 },
                history.notes,
                id,
            ],
        )?;
        
        Ok(())
    }
    
    fn row_to_employment_history(row: &Row) -> rusqlite::Result<EmploymentHistory> {
        Ok(EmploymentHistory {
            id: Some(row.get(0)?),
            employee_id: row.get(1)?,
            hire_date: {
                let date_str: String = row.get(2)?;
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| rusqlite::Error::InvalidQuery)?
            },
            termination_date: {
                let date_opt: Option<String> = row.get(3)?;
                date_opt.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
            },
            termination_reason: row.get::<_, Option<String>>(4)?,
            rehire_eligible: {
                let eligible: i32 = row.get(5)?;
                eligible != 0
            },
            notes: row.get::<_, Option<String>>(6)?,
            created_at: {
                let dt_str: String = row.get(7)?;
                DateTime::parse_from_rfc3339(&dt_str).map_err(|_| rusqlite::Error::InvalidQuery)?.with_timezone(&Utc)
            },
        })
    }
}
