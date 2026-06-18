use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

use cpr_core::models::{RegistryEntry, RegistryValue};
use crate::DbResult;

pub struct RegistryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl RegistryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Set a value at the given key path (creates or updates)
    pub fn set(&self, key_path: &str, value: &RegistryValue) -> DbResult<()> {
        let entry = RegistryEntry::new(key_path.to_string(), value.clone());
        entry.validate()?;

        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();

        match value {
            RegistryValue::String(s) => {
                conn.execute(
                    "INSERT INTO registry (key_path, value_type, value_string, created_at, updated_at)
                     VALUES (?1, 'String', ?2, ?3, ?4)
                     ON CONFLICT(key_path) DO UPDATE SET
                     value_type = 'String', value_string = ?2, value_integer = NULL, updated_at = ?4",
                    params![key_path, s, now, now],
                )?;
            }
            RegistryValue::Integer(i) => {
                conn.execute(
                    "INSERT INTO registry (key_path, value_type, value_integer, created_at, updated_at)
                     VALUES (?1, 'Integer', ?2, ?3, ?4)
                     ON CONFLICT(key_path) DO UPDATE SET
                     value_type = 'Integer', value_integer = ?2, value_string = NULL, updated_at = ?4",
                    params![key_path, i, now, now],
                )?;
            }
            RegistryValue::Boolean(b) => {
                let bool_str = if *b { "true" } else { "false" };
                conn.execute(
                    "INSERT INTO registry (key_path, value_type, value_string, created_at, updated_at)
                     VALUES (?1, 'Boolean', ?2, ?3, ?4)
                     ON CONFLICT(key_path) DO UPDATE SET
                     value_type = 'Boolean', value_string = ?2, value_integer = NULL, updated_at = ?4",
                    params![key_path, bool_str, now, now],
                )?;
            }
            RegistryValue::Json(j) => {
                let json_str = j.to_string();
                conn.execute(
                    "INSERT INTO registry (key_path, value_type, value_string, created_at, updated_at)
                     VALUES (?1, 'Json', ?2, ?3, ?4)
                     ON CONFLICT(key_path) DO UPDATE SET
                     value_type = 'Json', value_string = ?2, value_integer = NULL, updated_at = ?4",
                    params![key_path, json_str, now, now],
                )?;
            }
        }

        Ok(())
    }

    /// Get a value by key path
    pub fn get(&self, key_path: &str) -> DbResult<Option<RegistryEntry>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT id, key_path, value_type, value_string, value_integer, created_at, updated_at
             FROM registry WHERE key_path = ?1",
            params![key_path],
            |row| {
                let id: i64 = row.get(0)?;
                let key_path: String = row.get(1)?;
                let value_type: String = row.get(2)?;
                let value_string: Option<String> = row.get(3)?;
                let value_integer: Option<i64> = row.get(4)?;
                let created_at: String = row.get(5)?;
                let updated_at: String = row.get(6)?;

                let value = match value_type.as_str() {
                    "String" => RegistryValue::String(value_string.unwrap_or_default()),
                    "Integer" => RegistryValue::Integer(value_integer.unwrap_or(0)),
                    "Boolean" => {
                        let bool_str = value_string.unwrap_or_else(|| "false".to_string());
                        RegistryValue::Boolean(bool_str == "true")
                    }
                    "Json" => {
                        let json_str = value_string.unwrap_or_else(|| "{}".to_string());
                        let json_val: serde_json::Value = serde_json::from_str(&json_str)
                            .unwrap_or(serde_json::Value::Null);
                        RegistryValue::Json(json_val)
                    }
                    _ => RegistryValue::String(value_string.unwrap_or_default()),
                };

                Ok(RegistryEntry {
                    id: Some(id),
                    key_path,
                    value,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                        .unwrap_or_else(|_| chrono::Utc::now().into())
                        .with_timezone(&chrono::Utc),
                })
            },
        );

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a key path
    pub fn delete(&self, key_path: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM registry WHERE key_path = ?1", params![key_path])?;
        Ok(())
    }

    /// Check if a key path exists
    pub fn exists(&self, key_path: &str) -> DbResult<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM registry WHERE key_path = ?1",
            params![key_path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// List all keys under a given path prefix
    pub fn list_keys(&self, path_prefix: &str) -> DbResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key_path FROM registry WHERE key_path LIKE ?1 ORDER BY key_path"
        )?;

        let pattern = format!("{}%", path_prefix);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row?);
        }
        Ok(keys)
    }

    /// Get all values under a given path prefix
    pub fn get_all(&self, path_prefix: &str) -> DbResult<Vec<RegistryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, key_path, value_type, value_string, value_integer, created_at, updated_at
             FROM registry WHERE key_path LIKE ?1 ORDER BY key_path"
        )?;

        let pattern = format!("{}%", path_prefix);
        let rows = stmt.query_map(params![pattern], |row| {
            let id: i64 = row.get(0)?;
            let key_path: String = row.get(1)?;
            let value_type: String = row.get(2)?;
            let value_string: Option<String> = row.get(3)?;
            let value_integer: Option<i64> = row.get(4)?;
            let created_at: String = row.get(5)?;
            let updated_at: String = row.get(6)?;

            let value = match value_type.as_str() {
                "String" => RegistryValue::String(value_string.unwrap_or_default()),
                "Integer" => RegistryValue::Integer(value_integer.unwrap_or(0)),
                "Boolean" => {
                    let bool_str = value_string.unwrap_or_else(|| "false".to_string());
                    RegistryValue::Boolean(bool_str == "true")
                }
                "Json" => {
                    let json_str = value_string.unwrap_or_else(|| "{}".to_string());
                    let json_val: serde_json::Value = serde_json::from_str(&json_str)
                        .unwrap_or(serde_json::Value::Null);
                    RegistryValue::Json(json_val)
                }
                _ => RegistryValue::String(value_string.unwrap_or_default()),
            };

            Ok(RegistryEntry {
                id: Some(id),
                key_path,
                value,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    /// Delete all keys under a given path prefix
    pub fn delete_all(&self, path_prefix: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("{}%", path_prefix);
        conn.execute("DELETE FROM registry WHERE key_path LIKE ?1", params![pattern])?;
        Ok(())
    }
}
