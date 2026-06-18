use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

use cpr_core::models::{Company, Province};
use crate::DbResult;

pub struct CompanyRepository {
    conn: Arc<Mutex<Connection>>,
}

impl CompanyRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, company: &Company) -> DbResult<()> {
        company.validate()?;
        
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO company (id, name, business_number, address, province, created_at) VALUES (1, ?1, ?2, ?3, ?4, ?5)",
            params![
                company.name,
                company.business_number.as_ref().map(|s| s.as_str()),
                company.address,
                company.province.code(),
                company.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update(&self, company: &Company) -> DbResult<()> {
        company.validate()?;
        
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE company SET name = ?1, business_number = ?2, address = ?3, province = ?4 WHERE id = ?5",
            params![
                company.name,
                company.business_number.as_ref().map(|s| s.as_str()),
                company.address,
                company.province.code(),
                company.id,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self) -> DbResult<Option<Company>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, business_number, address, province, created_at
             FROM company WHERE id = 1"
        )?;
        
        let result = stmt.query_row([], |row| {
            Ok(Company {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                business_number: row.get(2)?,
                address: row.get(3)?,
                province: Province::from_code(&row.get::<_, String>(4)?)
                    .ok_or(rusqlite::Error::InvalidQuery)?,
                created_at: row.get(5)?,
            })
        });
        
        match result {
            Ok(company) => Ok(Some(company)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert(&self, company: &Company) -> DbResult<()> {
        // Check if company exists
        if self.get()?.is_some() {
            self.update(company)
        } else {
            self.create(company)
        }
    }
}
