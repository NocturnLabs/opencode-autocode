use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::schema;
use super::{FeatureRepository, KnowledgeRepository, MetaRepository, SessionRepository};

/// Default database filename
pub const DEFAULT_DB_PATH: &str = ".forger/progress.db";

/// Database connection wrapper with thread-safe access
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path
    ///
    /// Enables WAL (Write-Ahead Logging) mode for better concurrency in parallel
    /// worker scenarios. WAL mode allows multiple readers with a single writer
    /// without locking contention.
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create database directory: {}", parent.display())
                })?;
            }
        }

        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database: {}", path.display()))?;

        // Set busy timeout to handle transient locks from parallel workers
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;

        // Enable WAL mode for better concurrency (multiple readers, single writer)
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

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

    /// Initialize database schema and run migrations
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(schema::SCHEMA)
            .context("Failed to initialize database schema")?;

        // Run migrations for existing databases (safe to run multiple times)
        // ALTER TABLE ADD COLUMN fails if column exists, which we ignore
        let _ = conn.execute_batch(schema::MIGRATION_ADD_LAST_ERROR);

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
}
