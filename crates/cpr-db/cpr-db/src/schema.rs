use rusqlite::Connection;
use crate::DbResult;

/// Initialize the complete database schema from embedded full schema SQL (idempotent with IF NOT EXISTS)
pub fn initialize_database(conn: &mut Connection) -> DbResult<()> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    conn.execute_batch(include_str!("schema.sql"))?;

    // --- migrations for existing databases ---
    // Add days_taken to vacation_time_off
    add_column_if_missing(conn, "vacation_time_off", "days_taken", "INTEGER NOT NULL DEFAULT 0")?;

    Ok(())
}

/// Helper: add a column to a table only if it doesn't already exist
fn add_column_if_missing(conn: &Connection, table: &str, column: &str, definition: &str) -> DbResult<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns: Vec<String> = stmt.query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    if !columns.contains(&column.to_string()) {
        conn.execute(&format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition), [])?;
    }
    Ok(())
}
