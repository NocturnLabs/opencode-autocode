//! Metadata repository for key-value storage
//!
//! Used to store persistent state like Discord message IDs.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Repository for key-value metadata
pub struct MetaRepository {
    conn: Arc<Mutex<Connection>>,
}

impl MetaRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;

        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set a value by key (insert or replace)
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            params![key, value],
        )
        .context("Failed to set metadata")?;
        Ok(())
    }

    /// Check if the project is marked as initialized
    pub fn is_initialized(&self) -> Result<bool> {
        Ok(self.get("initialization_complete")? == Some("true".to_string()))
    }

    /// Mark the project as initialized
    pub fn mark_initialized(&self) -> Result<()> {
        self.set("initialization_complete", "true")
    }
}
