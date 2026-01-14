use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::schema;
use super::{FeatureRepository, KnowledgeRepository, MetaRepository, SessionRepository};

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

        let mut conn = open_connection(path)?;
        if let Err(err) = initialize_connection(&conn) {
            if is_corrupt_database_error_message(&err.to_string()) {
                drop(conn);
                let backup_path = backup_corrupt_database(path)?;
                eprintln!(
                    "⚠️ Database corruption detected. Backed up to {} and recreating.",
                    backup_path.display()
                );
                conn = open_connection(path)
                    .with_context(|| format!("Failed to recreate database: {}", path.display()))?;
                initialize_connection(&conn)?;
            } else {
                return Err(err);
            }
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
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

/// @description Opens a SQLite connection without running migrations.
/// @param path The database file path.
fn open_connection(path: &Path) -> Result<Connection> {
    Connection::open(path).with_context(|| format!("Failed to open database: {}", path.display()))
}

/// @description Initializes SQLite connection settings and schema.
/// @param conn The connection to configure.
fn initialize_connection(conn: &Connection) -> Result<()> {
    // Set busy timeout to handle transient locks from parallel workers
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;

    // Enable WAL mode for better concurrency (multiple readers, single writer)
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(schema::SCHEMA)
        .context("Failed to initialize database schema")?;

    // Run migrations for existing databases (safe to run multiple times)
    // ALTER TABLE ADD COLUMN fails if column exists, which we ignore
    let _ = conn.execute_batch(schema::MIGRATION_ADD_LAST_ERROR);

    Ok(())
}

/// @description Detects whether a database error message indicates corruption.
/// @param message Error message captured during database initialization.
fn is_corrupt_database_error_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("database disk image is malformed")
        || lower.contains("file is not a database")
        || lower.contains("database is malformed")
}

/// @description Back up a corrupt database file before reinitialization.
/// @param path The path to the corrupted database file.
/// @returns The backup path created for the corrupt database.
fn backup_corrupt_database(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_path = PathBuf::from(format!("{}.corrupt-{}", path.display(), timestamp));
    std::fs::rename(path, &backup_path).with_context(|| {
        format!(
            "Failed to back up corrupt database from {} to {}",
            path.display(),
            backup_path.display()
        )
    })?;
    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::Database;

    /// @description Restores a corrupt database by backing it up and recreating it.
    #[test]
    fn recovers_from_corrupt_database() {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("progress.db");
        std::fs::write(&db_path, b"corrupt-data").expect("write corrupt db");

        let db = Database::open(&db_path).expect("open db");
        let _ = db.features();

        let backup_exists = std::fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("progress.db.corrupt-")
            });

        assert!(backup_exists, "expected corrupt database backup");
        assert!(db_path.exists(), "expected recreated database file");
    }
}
