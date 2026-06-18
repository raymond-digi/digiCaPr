use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::{DbError, DbResult};
use crate::password::{generate_hash, verify_hash};

pub struct ConfigRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ConfigRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Set or update the main database password
    pub fn set_password(&self, password: &str, hint: Option<&str>) -> DbResult<()> {
        let hash = generate_hash(password)?;
        let hint_str = hint.unwrap_or("");

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO config (id, password_hash, password_hint, is_locked) VALUES (1, ?1, ?2, 1)
             ON CONFLICT(id) DO UPDATE SET
             password_hash = ?1, password_hint = ?2, is_locked = 1, updated_at = CURRENT_TIMESTAMP",
            params![hash, hint_str],
        )?;

        Ok(())
    }

    /// Set or update the master recovery key
    pub fn set_recovery_key(&self, recovery_key: &str) -> DbResult<()> {
        let hash = generate_hash(recovery_key)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET recovery_key_hash = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![hash],
        )?;

        Ok(())
    }

    /// Set backup email for recovery notifications
    pub fn set_backup_email(&self, email: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET backup_email = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![email],
        )?;

        Ok(())
    }

    /// Get password hint
    pub fn get_hint(&self) -> DbResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let hint: Option<String> = conn
            .query_row(
                "SELECT password_hint FROM config WHERE id = 1 AND password_hint != ''",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(hint)
    }

    /// Reset password using recovery key
    pub fn reset_password(&self, recovery_key: &str, new_password: &str, new_hint: Option<&str>) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        let recovery_hash: Option<String> = conn.query_row(
            "SELECT recovery_key_hash FROM config WHERE id = 1",
            [],
            |row| row.get(0),
        ).optional()?;

        if let Some(hash) = recovery_hash {
            if !verify_hash(&hash, recovery_key)? {
                return Err(DbError::InvalidData("Invalid recovery key".to_string()));
            }
        } else {
            return Err(DbError::InvalidData("No recovery key configured".to_string()));
        }

        // Set new password
        self.set_password(new_password, new_hint)?;

        Ok(())
    }

    /// Remove password protection
    pub fn remove_password(&self) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET password_hash = NULL, is_locked = 0, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            [],
        )?;

        Ok(())
    }

    /// Get master support email
    pub fn get_support_email(&self) -> DbResult<String> {
        let conn = self.conn.lock().unwrap();
        let email: String = conn.query_row(
            "SELECT COALESCE(support_email, 'support@cpr.com') FROM config WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(email)
    }

    /// Set master support email (admin only)
    pub fn set_support_email(&self, email: &str) -> DbResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET support_email = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![email],
        )?;

        Ok(())
    }
}