//! SQLite database module for progress tracking
//!
//! Provides persistent storage for features, sessions, and audit logs.
//! Replaces the previous file-based tracking (feature_list.json).

// Allow dead code for internal APIs that will be used by autonomous loop integration
#![allow(dead_code)]

mod features;
mod schema;
mod sessions;

// Re-export types used by main.rs
pub use features::FeatureRepository;
pub use sessions::SessionRepository;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Default database filename
pub const DEFAULT_DB_PATH: &str = ".opencode.db";

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

    /// Check if database exists at path
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Check if default database exists
    pub fn default_exists() -> bool {
        Self::exists(Path::new(DEFAULT_DB_PATH))
    }
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
