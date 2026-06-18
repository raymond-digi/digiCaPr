use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use rust_decimal::Decimal;

use crate::currency::{decimal_to_cents, convert_cents};
use crate::{DbResult, DbError};
use cpr_core::models::PersonalAmount;
use cpr_core::tax::config::load_tax_config;

pub struct PersonalAmountRepository {
    conn: Arc<Mutex<Connection>>,
}

impl PersonalAmountRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, pa: &mut PersonalAmount) -> DbResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO personal_amount (
                employee_id, province, year, federal_amount, provincial_amount, indexed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                pa.employee_id,
                &pa.province,
                pa.year,
                decimal_to_cents(&pa.federal_amount)?,
                decimal_to_cents(&pa.provincial_amount)?,
                pa.indexed_at.to_rfc3339(),
            ],
        )?;
        let id = conn.last_insert_rowid();
        pa.id = Some(id);
        Ok(id)
    }

    pub fn get(&self, id: i64) -> DbResult<PersonalAmount> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, province, year, federal_amount, provincial_amount, indexed_at
             FROM personal_amount WHERE id = ?1"
        )?;
        let pa = stmt.query_row([id], |row| {
            Ok(PersonalAmount {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                province: row.get(2)?,
                year: row.get(3)?,
                federal_amount: convert_cents(row.get::<_, i64>(4)?, "federal_amount")?,
                provincial_amount: convert_cents(row.get::<_, i64>(5)?, "provincial_amount")?,
                indexed_at: row.get(6)?,
            })
        })?;
        Ok(pa)
    }

    pub fn update(&self, id: i64, pa: &PersonalAmount) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE personal_amount SET
                federal_amount = ?1,
                provincial_amount = ?2,
                indexed_at = ?3
             WHERE id = ?4",
            params![
                decimal_to_cents(&pa.federal_amount)?,
                decimal_to_cents(&pa.provincial_amount)?,
                pa.indexed_at.to_rfc3339(),
                id,
            ],
        )?;
        Ok(())
    }

    pub fn list_by_employee(&self, employee_id: i64) -> DbResult<Vec<PersonalAmount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, province, year, federal_amount, provincial_amount, indexed_at
             FROM personal_amount WHERE employee_id = ?1 ORDER BY year DESC"
        )?;
        let rows = stmt.query_map([employee_id], |row| {
            Ok(PersonalAmount {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                province: row.get(2)?,
                year: row.get(3)?,
                federal_amount: convert_cents(row.get::<_, i64>(4)?, "federal_amount")?,
                provincial_amount: convert_cents(row.get::<_, i64>(5)?, "provincial_amount")?,
                indexed_at: row.get(6)?,
            })
        })?;
        let mut pas = Vec::new();
        for row in rows {
            pas.push(row?);
        }
        Ok(pas)
    }

    pub fn get_by_employee_province_year(&self, employee_id: i64, province: &str, year: i32) -> DbResult<Option<PersonalAmount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, province, year, federal_amount, provincial_amount, indexed_at
             FROM personal_amount WHERE employee_id = ?1 AND province = ?2 AND year = ?3"
        )?;
        let result = stmt.query_row(params![employee_id, province, year], |row| {
            Ok(PersonalAmount {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                province: row.get(2)?,
                year: row.get(3)?,
                federal_amount: convert_cents(row.get::<_, i64>(4)?, "federal_amount")?,
                provincial_amount: convert_cents(row.get::<_, i64>(5)?, "provincial_amount")?,
                indexed_at: row.get(6)?,
            })
        });
        match result {
            Ok(pa) => Ok(Some(pa)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_latest_for_employee(&self, employee_id: i64) -> DbResult<Option<PersonalAmount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, province, year, federal_amount, provincial_amount, indexed_at
             FROM personal_amount WHERE employee_id = ?1 ORDER BY year DESC LIMIT 1"
        )?;
        let result = stmt.query_row(params![employee_id], |row| {
            Ok(PersonalAmount {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                province: row.get(2)?,
                year: row.get(3)?,
                federal_amount: convert_cents(row.get::<_, i64>(4)?, "federal_amount")?,
                provincial_amount: convert_cents(row.get::<_, i64>(5)?, "provincial_amount")?,
                indexed_at: row.get(6)?,
            })
        });
        match result {
            Ok(pa) => Ok(Some(pa)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_latest_by_employee_and_province(&self, employee_id: i64, province: &str) -> DbResult<Option<PersonalAmount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, province, year, federal_amount, provincial_amount, indexed_at
             FROM personal_amount WHERE employee_id = ?1 AND province = ?2 ORDER BY year DESC LIMIT 1"
        )?;
        let result = stmt.query_row(params![employee_id, province], |row| {
            Ok(PersonalAmount {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                province: row.get(2)?,
                year: row.get(3)?,
                federal_amount: convert_cents(row.get::<_, i64>(4)?, "federal_amount")?,
                provincial_amount: convert_cents(row.get::<_, i64>(5)?, "provincial_amount")?,
                indexed_at: row.get(6)?,
            })
        });
        match result {
            Ok(pa) => Ok(Some(pa)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get personal amount for a specific year, with indexing if needed.
    /// This method will:
    /// 1. Check if personal amounts exist for the target year in the database
    /// 2. If not found, get the latest available personal amounts from a previous year
    /// 3. Index those amounts to the target year using tax config data
    /// 4. Return error if no personal amounts are found in the database at all
    pub fn get_personal_amount_with_fallback(&self, employee_id: i64, province: &str, year: i32) -> DbResult<(Decimal, Decimal)> {
        // First try to get exact year match
        if let Some(pa) = self.get_by_employee_province_year(employee_id, province, year)? {
            return Ok((pa.federal_amount, pa.provincial_amount));
        }
        
        // If no exact match, get latest available personal amount and try to index it
        if let Some(source_pa) = self.get_latest_by_employee_and_province(employee_id, province)? {
            // If the source year is different from target year, we need to index
            if source_pa.year != year {
                // Try to load tax configs for both source and target years
                match (load_tax_config(source_pa.year), load_tax_config(year)) {
                    (Ok(source_config), Ok(target_config)) => {
                        // Get province-specific config
                        if let Some(province_enum) = cpr_core::models::Province::from_code(province) {
                            if let (Some(source_prov_config), Some(target_prov_config)) = (
                                source_config.provincial.province_configs.get(&province_enum),
                                target_config.provincial.province_configs.get(&province_enum)
                            ) {
                                // Calculate indexed amounts based on the ratio of basic personal amounts
                                // Federal indexing: ratio of target federal BPA to source federal BPA
                                let federal_ratio = target_config.federal.basic_personal_amount / source_config.federal.basic_personal_amount;
                                let indexed_federal = source_pa.federal_amount * federal_ratio;
                                
                                // Provincial indexing: ratio of target provincial BPA to source provincial BPA
                                let provincial_ratio = target_prov_config.basic_personal_amount / source_prov_config.basic_personal_amount;
                                let indexed_provincial = source_pa.provincial_amount * provincial_ratio;
                                
                                return Ok((indexed_federal, indexed_provincial));
                            }
                        }
                    }
                    _ => {
                        // Can't load tax configs - return error
                        return Err(DbError::InvalidData(format!(
                            "Cannot load tax config for year {} or {}. Tax config files are required for personal amount indexing.",
                            source_pa.year, year
                        )));
                    }
                }
            }
            
            // Use source amounts (either same year or fallback when indexing not possible)
            return Ok((source_pa.federal_amount, source_pa.provincial_amount));
        }
        
        // If no personal amounts available at all, return error (no default BPA fallback)
        Err(DbError::InvalidData(format!(
            "No personal amount found for employee {} in province {} for year {} or any previous year. Please configure personal amounts before calculating payroll.",
            employee_id, province, year
        )))
    }
}