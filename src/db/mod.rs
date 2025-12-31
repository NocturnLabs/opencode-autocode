//! SQLite database module for progress tracking
//!
//! Provides persistent storage for features, sessions, and audit logs.
//! Replaces the previous file-based tracking (feature_list.json).

// Allow dead code for internal APIs that will be used by autonomous loop integration
#![allow(dead_code)]

pub mod features;
mod schema;
mod sessions;
pub mod meta;
pub mod knowledge;

// Re-export types used by main.rs
pub use features::FeatureRepository;
pub use sessions::SessionRepository;
pub use meta::MetaRepository;
pub use knowledge::KnowledgeRepository;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Default database filename
pub const DEFAULT_DB_PATH: &str = ".autocode/progress.db";

/// Database connection wrapper with thread-safe access
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {}", path.display()))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    /// Open database with default path (.opencode.db in current directory)
    pub fn open_default() -> Result<Self> {
        Self::open(Path::new(DEFAULT_DB_PATH))
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(schema::SCHEMA)
            .context("Failed to initialize database schema")?;
        Ok(())
    }

    /// Get a reference to the connection (for repositories)
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Get feature repository
    pub fn features(&self) -> FeatureRepository {
        FeatureRepository::new(self.connection())
    }

    /// Get session repository
    pub fn sessions(&self) -> SessionRepository {
        SessionRepository::new(self.connection())
    }

    /// Get meta repository
    pub fn meta(&self) -> MetaRepository {
        MetaRepository::new(self.connection())
    }

    /// Get knowledge repository
    pub fn knowledge(&self) -> KnowledgeRepository {
        KnowledgeRepository::new(self.connection())
    }

    /// Check if database exists at path
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Check if default database exists
    pub fn default_exists() -> bool {
        Self::exists(Path::new(DEFAULT_DB_PATH))
    }

    // ============================================================
    // MCP-equivalent operations (replaces SQLite MCP server)
    // ============================================================

    /// Execute a read-only SELECT query, returns formatted output
    pub fn read_query(&self, sql: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows: Vec<Vec<String>> = stmt
            .query_map([], |row| {
                let mut values = Vec::new();
                for i in 0..column_count {
                    let value: String = row
                        .get::<_, rusqlite::types::Value>(i)
                        .map(|v| format_value(&v))
                        .unwrap_or_else(|_| "NULL".to_string());
                    values.push(value);
                }
                Ok(values)
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Format as table
        Ok(format_table(&column_names, &rows))
    }

    /// Execute a write query (INSERT, UPDATE, DELETE, CREATE), returns rows affected
    pub fn write_query(&self, sql: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(sql, [])?;
        Ok(affected)
    }

    /// List all table names in the database
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tables)
    }

    /// Get the schema (CREATE statement) for a table
    pub fn describe_table(&self, table_name: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();

        // Get column info
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns: Vec<(String, String, bool, bool)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(1)?,   // name
                    row.get::<_, String>(2)?,   // type
                    row.get::<_, i32>(3)? != 0, // notnull
                    row.get::<_, i32>(5)? != 0, // pk
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if columns.is_empty() {
            anyhow::bail!("Table '{}' not found", table_name);
        }

        let mut output = format!("Table: {}\n", table_name);
        output.push_str("Columns:\n");
        for (name, col_type, notnull, pk) in columns {
            let mut flags = Vec::new();
            if pk {
                flags.push("PRIMARY KEY");
            }
            if notnull {
                flags.push("NOT NULL");
            }
            let flags_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };
            output.push_str(&format!("  - {}: {}{}\n", name, col_type, flags_str));
        }

        Ok(output)
    }
}

/// Format a SQLite value as a string (DRY helper)
fn format_value(value: &rusqlite::types::Value) -> String {
    match value {
        rusqlite::types::Value::Null => "NULL".to_string(),
        rusqlite::types::Value::Integer(i) => i.to_string(),
        rusqlite::types::Value::Real(f) => f.to_string(),
        rusqlite::types::Value::Text(s) => s.clone(),
        rusqlite::types::Value::Blob(_) => "[BLOB]".to_string(),
    }
}

/// Format rows as a simple table (DRY helper)
fn format_table(headers: &[String], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(no rows)\n".to_string();
    }

    // Calculate column widths
    let col_widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            let header_len = headers[i].len();
            let max_row_len = rows
                .iter()
                .map(|r| r.get(i).map_or(0, |s| s.len()))
                .max()
                .unwrap_or(0);
            header_len.max(max_row_len)
        })
        .collect();

    let mut output = String::new();

    // Header
    for (i, h) in headers.iter().enumerate() {
        output.push_str(&format!("{:width$}", h, width = col_widths[i] + 2));
    }
    output.push('\n');

    // Separator
    for w in &col_widths {
        output.push_str(&format!("{:-<width$}", "", width = w + 2));
    }
    output.push('\n');

    // Rows
    for row in rows {
        for (i, v) in row.iter().enumerate() {
            output.push_str(&format!("{:width$}", v, width = col_widths[i] + 2));
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_open_creates_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::open(&db_path).unwrap();
        assert!(db_path.exists());

        // Verify schema was created
        let conn = db.conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='features'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_open_default() {
        // This test would create a file in the current directory, so we skip it
        // in favor of the path-based test above
    }
}
