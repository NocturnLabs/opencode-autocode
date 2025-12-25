//! Session repository for tracking autonomous runs
//!
//! Provides session management and event logging.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Session data structure
#[derive(Debug, Clone)]
pub struct Session {
    pub id: i64,
    pub session_number: i32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub features_before: i32,
    pub features_after: i32,
    pub status: String,
}

/// Session event data
#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub id: i64,
    pub session_id: i64,
    pub event_type: String,
    pub message: Option<String>,
    pub timestamp: String,
}

/// Event types for session logging
pub mod event_types {
    pub const STARTED: &str = "started";
    pub const FEATURE_COMPLETED: &str = "feature_completed";
    pub const ERROR: &str = "error";
    pub const TIMEOUT: &str = "timeout";
    pub const STOPPED: &str = "stopped";
    pub const COMPLETED: &str = "completed";
}

/// Session status values
pub mod status {
    pub const RUNNING: &str = "running";
    pub const COMPLETED: &str = "completed";
    pub const FAILED: &str = "failed";
    pub const STOPPED: &str = "stopped";
}

/// Repository for session operations
pub struct SessionRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SessionRepository {
    /// Create a new repository with the given connection
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Start a new session
    pub fn start_session(&self, session_number: i32, features_before: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO sessions (session_number, features_before, status)
             VALUES (?1, ?2, ?3)",
            params![session_number, features_before, status::RUNNING],
        )
        .context("Failed to start session")?;

        let session_id = conn.last_insert_rowid();

        // Log session start event
        conn.execute(
            "INSERT INTO session_events (session_id, event_type, message)
             VALUES (?1, ?2, ?3)",
            params![
                session_id,
                event_types::STARTED,
                format!(
                    "Session {} started with {} features",
                    session_number, features_before
                )
            ],
        )
        .context("Failed to log session start")?;

        Ok(session_id)
    }

    /// End a session
    pub fn end_session(&self, session_id: i64, features_after: i32, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE sessions SET completed_at = datetime('now'), features_after = ?1, status = ?2
             WHERE id = ?3",
            params![features_after, status, session_id],
        )
        .context("Failed to end session")?;

        Ok(())
    }

    /// Log an event for a session
    pub fn log_event(
        &self,
        session_id: i64,
        event_type: &str,
        message: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO session_events (session_id, event_type, message)
             VALUES (?1, ?2, ?3)",
            params![session_id, event_type, message],
        )
        .context("Failed to log event")?;

        Ok(())
    }

    /// Get the next session number
    pub fn next_session_number(&self) -> Result<i32> {
        let conn = self.conn.lock().unwrap();

        let max: Option<i32> = conn
            .query_row("SELECT MAX(session_number) FROM sessions", [], |row| {
                row.get(0)
            })
            .context("Failed to get max session number")?;

        Ok(max.unwrap_or(0) + 1)
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: i64) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT id, session_number, started_at, completed_at, features_before, features_after, status
             FROM sessions WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    session_number: row.get(1)?,
                    started_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    features_before: row.get(4)?,
                    features_after: row.get(5)?,
                    status: row.get(6)?,
                })
            },
        );

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get session"),
        }
    }

    /// Get all sessions
    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, session_number, started_at, completed_at, features_before, features_after, status
                 FROM sessions ORDER BY id DESC",
            )
            .context("Failed to prepare session query")?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    session_number: row.get(1)?,
                    started_at: row.get(2)?,
                    completed_at: row.get(3)?,
                    features_before: row.get(4)?,
                    features_after: row.get(5)?,
                    status: row.get(6)?,
                })
            })
            .context("Failed to query sessions")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read sessions")?;

        Ok(sessions)
    }

    /// Get events for a session
    pub fn get_events(&self, session_id: i64) -> Result<Vec<SessionEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, event_type, message, timestamp
                 FROM session_events WHERE session_id = ?1 ORDER BY id",
            )
            .context("Failed to prepare event query")?;

        let events = stmt
            .query_map(params![session_id], |row| {
                Ok(SessionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    event_type: row.get(2)?,
                    message: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .context("Failed to query events")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read events")?;

        Ok(events)
    }

    /// Get session statistics
    pub fn get_stats(&self) -> Result<SessionStats> {
        let conn = self.conn.lock().unwrap();

        let total_sessions: i32 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .context("Failed to count sessions")?;

        let completed_sessions: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE status = ?1",
                params![status::COMPLETED],
                |row| row.get(0),
            )
            .context("Failed to count completed sessions")?;

        let total_features_completed: i32 = conn
            .query_row(
                "SELECT COALESCE(SUM(features_after - features_before), 0) FROM sessions
                 WHERE features_after > features_before",
                [],
                |row| row.get(0),
            )
            .context("Failed to sum features completed")?;

        Ok(SessionStats {
            total_sessions: total_sessions as usize,
            completed_sessions: completed_sessions as usize,
            total_features_completed: total_features_completed as usize,
        })
    }
}

/// Session statistics summary
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub total_features_completed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::TempDir;

    fn setup_test_db() -> (TempDir, Database) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        (temp_dir, db)
    }

    #[test]
    fn test_session_lifecycle() {
        let (_temp, db) = setup_test_db();
        let repo = db.sessions();

        // Start session
        let session_id = repo.start_session(1, 10).unwrap();
        assert!(session_id > 0);

        // Check session exists
        let session = repo.get_session(session_id).unwrap().unwrap();
        assert_eq!(session.session_number, 1);
        assert_eq!(session.status, "running");

        // Log an event
        repo.log_event(
            session_id,
            event_types::FEATURE_COMPLETED,
            Some("Test feature"),
        )
        .unwrap();

        // End session
        repo.end_session(session_id, 12, status::COMPLETED).unwrap();

        // Verify session updated
        let session = repo.get_session(session_id).unwrap().unwrap();
        assert_eq!(session.status, "completed");
        assert_eq!(session.features_after, 12);
    }

    #[test]
    fn test_next_session_number() {
        let (_temp, db) = setup_test_db();
        let repo = db.sessions();

        assert_eq!(repo.next_session_number().unwrap(), 1);

        repo.start_session(1, 0).unwrap();
        assert_eq!(repo.next_session_number().unwrap(), 2);

        repo.start_session(2, 0).unwrap();
        assert_eq!(repo.next_session_number().unwrap(), 3);
    }
}
