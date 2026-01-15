//! Session repository for tracking autonomous runs
//!
//! Provides session management and event logging.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Session status values
pub mod status {
    pub const COMPLETED: &str = "completed";
}

/// Repository for session operations
pub struct SessionRepository {
    conn: Arc<Mutex<Connection>>,
}

/// Session data for API responses
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct Session {
    pub id: i64,
    pub session_number: i32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub features_before: i32,
    pub features_after: i32,
    pub status: String,
}

/// Session event data for API responses
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct SessionEvent {
    pub id: i64,
    pub session_id: i64,
    pub event_type: String,
    pub message: Option<String>,
    pub timestamp: String,
}

impl SessionRepository {
    /// Create a new repository with the given connection
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// List all sessions, ordered by most recent first
    #[allow(dead_code)]
    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_number, started_at, completed_at, features_before, features_after, status
             FROM sessions ORDER BY started_at DESC"
        )?;

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
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(sessions)
    }

    /// Get a session by ID with its events
    #[allow(dead_code)]
    pub fn get_session_with_events(
        &self,
        session_id: i64,
    ) -> Result<Option<(Session, Vec<SessionEvent>)>> {
        let conn = self.conn.lock().unwrap();

        // Get session
        let session: Option<Session> = conn
            .query_row(
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
            )
            .ok();

        let Some(session) = session else {
            return Ok(None);
        };

        // Get events
        let mut stmt = conn.prepare(
            "SELECT id, session_id, event_type, message, timestamp
             FROM session_events WHERE session_id = ?1 ORDER BY timestamp ASC",
        )?;

        let events = stmt
            .query_map(params![session_id], |row| {
                Ok(SessionEvent {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    event_type: row.get(2)?,
                    message: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Some((session, events)))
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
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub total_features_completed: usize,
}

#[cfg(test)]
mod tests {
    use crate::db::test_utils::tests::setup_test_db;

    #[test]
    fn test_get_stats_empty() {
        let (_temp, db) = setup_test_db();
        let repo = db.sessions();

        let stats = repo.get_stats().unwrap();
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.completed_sessions, 0);
        assert_eq!(stats.total_features_completed, 0);
    }
}
