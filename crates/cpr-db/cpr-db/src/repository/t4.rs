use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use crate::{DbResult, DbError};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

// T4 box values can be negative (e.g., adjustment reduces below calculated value),
// so we need dedicated conversion helpers that allow negatives.
fn t4_decimal_to_cents(value: &Decimal) -> DbResult<i64> {
    let rounded = value.round_dp(2);
    let cents = (rounded * Decimal::new(100, 0)).round_dp(0);
    cents.to_i64().ok_or_else(|| DbError::InvalidData(format!("Value too large: {}", rounded)))
}

fn t4_cents_to_decimal(cents: i64) -> Decimal {
    Decimal::new(cents, 2)
}

// ============================================================================
// Flexible T4 Schema Models
// ============================================================================

/// T4 slip status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum T4SlipStatus {
    Draft,
    Calculated,
    Filed,
    Locked,
}

impl T4SlipStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            T4SlipStatus::Draft => "draft",
            T4SlipStatus::Calculated => "calculated",
            T4SlipStatus::Filed => "filed",
            T4SlipStatus::Locked => "locked",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(T4SlipStatus::Draft),
            "calculated" => Some(T4SlipStatus::Calculated),
            "filed" => Some(T4SlipStatus::Filed),
            "locked" => Some(T4SlipStatus::Locked),
            _ => None,
        }
    }
}

/// T4 slip record (replaces t4_calculated + t4_adjustments)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4SlipRecord {
    pub id: Option<i64>,
    pub employee_id: i64,
    pub year: i32,
    pub slip_version: i32,
    pub status: T4SlipStatus,
    /// Sum of net_pay from payroll history (ground truth, in cents)
    pub net_pay: Decimal,
    pub filed_at: Option<DateTime<Utc>>,
    pub filed_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// T4 box value (flexible key-value storage for box values)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4BoxValue {
    pub id: Option<i64>,
    pub t4_slip_id: i64,
    pub box_type: String,
    pub calculated_value: Decimal,
    pub adjustment_value: Decimal,
}

impl T4BoxValue {
    /// Get the final value (calculated + adjustment)
    pub fn final_value(&self) -> Decimal {
        self.calculated_value + self.adjustment_value
    }
}

/// T4 slip data with all box values (for UI display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T4SlipData {
    pub slip: T4SlipRecord,
    pub box_values: Vec<T4BoxValue>,
}

pub struct T4Repository {
    conn: Arc<Mutex<Connection>>,
}

impl T4Repository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get T4 years that have slips
    pub fn get_t4_years(&self) -> DbResult<Vec<i32>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT year FROM t4_slip ORDER BY year DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            row.get(0)
        })?;
        let mut years = Vec::new();
        for row_result in rows {
            years.push(row_result?);
        }
        Ok(years)
    }

    /// Get or create a T4 slip for an employee/year with version tracking
    pub fn get_or_create_slip(&self, employee_id: i64, year: i32) -> DbResult<T4SlipRecord> {
        let conn = self.conn.lock().unwrap();
        
        // Get latest version
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, year, slip_version, status, net_pay, filed_at, filed_by, created_at, updated_at
             FROM t4_slip WHERE employee_id = ?1 AND year = ?2
             ORDER BY slip_version DESC LIMIT 1"
        )?;
        let result = stmt.query_row(params![employee_id, year], |row| {
            Ok(T4SlipRecord {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                year: row.get(2)?,
                slip_version: row.get(3)?,
                status: T4SlipStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(T4SlipStatus::Draft),
                net_pay: t4_cents_to_decimal(row.get::<_, i64>(5)?),
                filed_at: None,
                filed_by: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
            })
        });
        match result {
            Ok(slip) => Ok(slip),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // Create new slip with version 1
                let now = Utc::now();
                let mut slip = T4SlipRecord {
                    id: None,
                    employee_id,
                    year,
                    slip_version: 1,
                    status: T4SlipStatus::Draft,
                    net_pay: Decimal::ZERO,
                    filed_at: None,
                    filed_by: None,
                    created_at: now,
                    updated_at: now,
                };
                // Save it
                conn.execute(
                    "INSERT INTO t4_slip (employee_id, year, slip_version, status, net_pay, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        slip.employee_id,
                        slip.year,
                        slip.slip_version,
                        slip.status.as_str(),
                        t4_decimal_to_cents(&slip.net_pay)?,
                        slip.created_at.to_rfc3339(),
                        slip.updated_at.to_rfc3339(),
                    ],
                )?;
                slip.id = Some(conn.last_insert_rowid());
                Ok(slip)
            }
            Err(e) => Err(e.into()),
        }
    }
    
    /// Create a new version of T4 slip for recalculation/amendment
    pub fn create_slip_version(&self, employee_id: i64, year: i32) -> DbResult<T4SlipRecord> {
        let conn = self.conn.lock().unwrap();
        
        // Get current max version
        let mut stmt = conn.prepare(
            "SELECT COALESCE(MAX(slip_version), 0) FROM t4_slip WHERE employee_id = ?1 AND year = ?2"
        )?;
        let max_version: i32 = stmt.query_row(params![employee_id, year], |row| row.get(0))?;
        let new_version = max_version + 1;
        
        let now = Utc::now();
        let mut slip = T4SlipRecord {
            id: None,
            employee_id,
            year,
            slip_version: new_version,
            status: T4SlipStatus::Draft,
            net_pay: Decimal::ZERO,
            filed_at: None,
            filed_by: None,
            created_at: now,
            updated_at: now,
        };
        
        conn.execute(
            "INSERT INTO t4_slip (employee_id, year, slip_version, status, net_pay, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                slip.employee_id,
                slip.year,
                slip.slip_version,
                slip.status.as_str(),
                t4_decimal_to_cents(&slip.net_pay)?,
                slip.created_at.to_rfc3339(),
                slip.updated_at.to_rfc3339(),
            ],
        )?;
        slip.id = Some(conn.last_insert_rowid());
        
        Ok(slip)
    }
    
    /// Get all slips for a year
    pub fn list_slips_by_year(&self, year: i32) -> DbResult<Vec<T4SlipRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, employee_id, year, slip_version, status, net_pay, filed_at, filed_by, created_at, updated_at
             FROM t4_slip WHERE year = ?1 ORDER BY employee_id, slip_version DESC"
        )?;
        let rows = stmt.query_map(params![year], |row| {
            Ok(T4SlipRecord {
                id: Some(row.get(0)?),
                employee_id: row.get(1)?,
                year: row.get(2)?,
                slip_version: row.get(3)?,
                status: T4SlipStatus::from_str(&row.get::<_, String>(4)?).unwrap_or(T4SlipStatus::Draft),
                net_pay: t4_cents_to_decimal(row.get::<_, i64>(5)?),
                filed_at: None,
                filed_by: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                    .map_err(|_| rusqlite::Error::InvalidQuery)?
                    .with_timezone(&Utc),
            })
        })?;
        let mut slips = Vec::new();
        for row_result in rows {
            slips.push(row_result?);
        }
        Ok(slips)
    }
    
    /// Save a box value (calculated or adjustment)
    pub fn save_box_value(&self, box_val: &mut T4BoxValue) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        if box_val.id.is_none() {
            conn.execute(
                "INSERT INTO t4_box_value (t4_slip_id, box_type, calculated_value, adjustment_value)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    box_val.t4_slip_id,
                    box_val.box_type,
                    t4_decimal_to_cents(&box_val.calculated_value)?,
                    t4_decimal_to_cents(&box_val.adjustment_value)?,
                ],
            )?;
            box_val.id = Some(conn.last_insert_rowid());
        } else {
            conn.execute(
                "UPDATE t4_box_value SET calculated_value = ?2, adjustment_value = ?3 WHERE id = ?1",
                params![
                    box_val.id.unwrap(),
                    t4_decimal_to_cents(&box_val.calculated_value)?,
                    t4_decimal_to_cents(&box_val.adjustment_value)?,
                ],
            )?;
        }
        Ok(())
    }
    
    /// Get all box values for a slip
    pub fn get_box_values(&self, slip_id: i64) -> DbResult<Vec<T4BoxValue>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, t4_slip_id, box_type, calculated_value, adjustment_value
             FROM t4_box_value WHERE t4_slip_id = ?1 ORDER BY box_type"
        )?;
        let rows = stmt.query_map(params![slip_id], |row| {
            Ok(T4BoxValue {
                id: Some(row.get(0)?),
                t4_slip_id: row.get(1)?,
                box_type: row.get(2)?,
                calculated_value: t4_cents_to_decimal(row.get::<_, i64>(3)?),
                adjustment_value: t4_cents_to_decimal(row.get::<_, i64>(4)?),
            })
        })?;
        let mut values = Vec::new();
        for row_result in rows {
            values.push(row_result?);
        }
        Ok(values)
    }
    
    /// Calculate and save box values for a year
    pub fn calculate_and_save_for_year(&self, year: i32, _calculated_by: &str) -> DbResult<()> {
        use crate::repository::ytd::YtdRepository;
        let ytd_repo = YtdRepository::new(Arc::clone(&self.conn));
        
        let ytd_list = ytd_repo.list_by_year(year)?;
        
        for ytd in ytd_list {
            let gross_pay = ytd.gross_pay;
            let cpp = ytd.cpp;
            let cpp2 = ytd.cpp2;
            let ei = ytd.ei;
            let federal = ytd.federal_tax;
            let provincial = ytd.provincial_tax;
            let income_tax = federal + provincial;
            
            // Get or create slip
            let slip = self.get_or_create_slip(ytd.employee_id, year)?;
            let slip_id = slip.id.unwrap();
            
            // Delete existing box values for this slip to allow recalculation
            {
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "DELETE FROM t4_box_value WHERE t4_slip_id = ?1",
                    params![slip_id],
                )?;
            }
            
            // Calculate Box 24 (EI insurable earnings) and Box 26 (CPP pensionable earnings)
            let ei_insurable_earnings = gross_pay;
            let cpp_exemption = rust_decimal::Decimal::from(3500);
            let cpp_pensionable_earnings = (gross_pay - cpp_exemption).max(rust_decimal::Decimal::ZERO);

            // Save box values using T4BoxType enum
            let box_calcs = vec![
                ("box_14", gross_pay),
                ("box_16", cpp),
                ("box_16a", cpp2),
                ("box_18", ei),
                ("box_20", ytd.rpp_contributions),
                ("box_22", income_tax),
                ("box_24", ei_insurable_earnings),
                ("box_26", cpp_pensionable_earnings),
                ("box_52", ytd.pension_adjustment),
            ];
            
            for (box_type, value) in box_calcs {
                let mut box_val = T4BoxValue {
                    id: None,
                    t4_slip_id: slip_id,
                    box_type: box_type.to_string(),
                    calculated_value: value,
                    adjustment_value: Decimal::ZERO,
                };
                self.save_box_value(&mut box_val)?;
            }
            
            // Update slip with net_pay from YTD (ground truth) and status
            let net_pay_cents = t4_decimal_to_cents(&ytd.net_pay)?;
            {
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "UPDATE t4_slip SET status = ?1, net_pay = ?2, updated_at = ?3 WHERE id = ?4",
                    params![T4SlipStatus::Calculated.as_str(), net_pay_cents, Utc::now().to_rfc3339(), slip.id.unwrap()],
                )?;
            }
        }
        Ok(())
    }
    
    /// File a T4 slip
    pub fn file_slip(&self, slip_id: i64, filed_by: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        conn.execute(
            "UPDATE t4_slip SET status = ?1, filed_at = ?2, filed_by = ?3, updated_at = ?4 WHERE id = ?5",
            params![T4SlipStatus::Filed.as_str(), now.to_rfc3339(), filed_by, now.to_rfc3339(), slip_id],
        )?;
        Ok(())
    }
    
    /// Lock a T4 slip (archive)
    pub fn lock_slip(&self, slip_id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        conn.execute(
            "UPDATE t4_slip SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![T4SlipStatus::Locked.as_str(), now.to_rfc3339(), slip_id],
        )?;
        Ok(())
    }
    
    /// Unlock a T4 slip for amendment
    pub fn unlock_slip(&self, slip_id: i64) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        conn.execute(
            "UPDATE t4_slip SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![T4SlipStatus::Draft.as_str(), now.to_rfc3339(), slip_id],
        )?;
        Ok(())
    }

    /// Update T4 box values for a slip (applies adjustment diffs to existing box values)
    pub fn update_box_values(
        &self,
        employee_id: i64,
        year: i32,
        box_adjustments: &[(String, Decimal)],
    ) -> DbResult<i64> {
        // Get or create the slip for this employee/year
        let slip = self.get_or_create_slip(employee_id, year)?;
        let slip_id = slip.id.ok_or_else(|| DbError::InvalidData("Slip has no ID".to_string()))?;
        
        // Get existing box values
        let existing_boxes = self.get_box_values(slip_id)?;
        
        for (box_type, adj_value) in box_adjustments {
            if *adj_value == Decimal::ZERO {
                continue;
            }
            
            // Find existing box value or create new one
            let existing = existing_boxes.iter().find(|b| b.box_type == *box_type);
            
            let mut box_val = if let Some(existing) = existing {
                T4BoxValue {
                    id: existing.id,
                    t4_slip_id: slip_id,
                    box_type: box_type.clone(),
                    calculated_value: existing.calculated_value,
                    // Accumulate: add the new diff to any previous adjustment
                    adjustment_value: existing.adjustment_value + *adj_value,
                }
            } else {
                T4BoxValue {
                    id: None,
                    t4_slip_id: slip_id,
                    box_type: box_type.clone(),
                    calculated_value: Decimal::ZERO,
                    adjustment_value: *adj_value,
                }
            };
            
            self.save_box_value(&mut box_val)?;
        }
        
        // Update slip status if it was calculated
        if slip.status == T4SlipStatus::Calculated {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE t4_slip SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![T4SlipStatus::Draft.as_str(), Utc::now().to_rfc3339(), slip_id],
            )?;
        }
        
        Ok(slip_id)
    }
}
