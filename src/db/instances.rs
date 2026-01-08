//! Instance repository for tracking running processes
//!
//! Tracks active instances of the application (CLI, Web, Workers) for the Control Panel.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Represents a running instance of the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: i64,
    pub pid: u32,
    pub role: String,
    pub start_time: String,
    pub status: String,
    pub log_path: Option<String>,
    pub project_path: Option<String>,
    pub updated_at: String,
}

/// Repository for instance operations
pub struct InstanceRepository {
    // We open a new connection to the global DB for each repository instance.
    // In a real app, we might use a connection pool, but for a CLI/Tool this is fine.
    conn: Connection,
}

const INSTANCE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER NOT NULL,
    role TEXT NOT NULL,
    start_time TEXT DEFAULT (datetime('now')),
    status TEXT DEFAULT 'running',
    log_path TEXT,
    project_path TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TRIGGER IF NOT EXISTS update_instances_timestamp
    AFTER UPDATE ON instances
    FOR EACH ROW
BEGIN
    UPDATE instances SET updated_at = datetime('now') WHERE id = NEW.id;
END;
"#;

impl InstanceRepository {
    /// Open the global instance registry
    pub fn open() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_dir = std::path::Path::new(&home).join(".opencode");

        if !db_dir.exists() {
            std::fs::create_dir_all(&db_dir).context("Failed to create global config directory")?;
        }

        let db_path = db_dir.join("registry.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open global registry: {}", db_path.display()))?;

        // Enable WAL for concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Init schema
        conn.execute_batch(INSTANCE_SCHEMA)?;

        Ok(Self { conn })
    }

    /// Register a new instance
    pub fn register(&self, pid: u32, role: &str, log_path: Option<&str>) -> Result<i64> {
        let project_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();

        self.conn.execute(
            "INSERT INTO instances (pid, role, status, log_path, project_path) VALUES (?1, ?2, 'running', ?3, ?4)",
            params![pid, role, log_path, project_path],
        )
        .context("Failed to register instance")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Update the heartbeat/status of an instance
    pub fn heartbeat(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE instances SET updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Mark an instance as stopped
    pub fn mark_stopped(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE instances SET status = 'stopped' WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// List all instances (optionally filtering by status)
    pub fn list(&self, active_only: bool) -> Result<Vec<Instance>> {
        let mut sql =
            "SELECT id, pid, role, start_time, status, log_path, updated_at, project_path FROM instances"
                .to_string();

        if active_only {
            sql.push_str(" WHERE status = 'running'");
        }
        sql.push_str(" ORDER BY start_time DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let instances = stmt
            .query_map([], |row| {
                Ok(Instance {
                    id: row.get(0)?,
                    pid: row.get(1)?,
                    role: row.get(2)?,
                    start_time: row.get(3)?,
                    status: row.get(4)?,
                    log_path: row.get(5)?,
                    updated_at: row.get(6)?,
                    project_path: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(instances)
    }

    /// Get a specific instance
    pub fn get(&self, id: i64) -> Result<Option<Instance>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pid, role, start_time, status, log_path, updated_at, project_path FROM instances WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Instance {
                id: row.get(0)?,
                pid: row.get(1)?,
                role: row.get(2)?,
                start_time: row.get(3)?,
                status: row.get(4)?,
                log_path: row.get(5)?,
                updated_at: row.get(6)?,
                project_path: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Prune old/stale instances (optional cleanup utility)
    #[allow(dead_code)]
    pub fn prune_stale(&self, hours: u32) -> Result<usize> {
        let affected = self.conn.execute(
            "DELETE FROM instances WHERE status = 'stopped' AND start_time < datetime('now', ?1)",
            params![format!("-{} hours", hours)],
        )?;
        Ok(affected)
    }
}
