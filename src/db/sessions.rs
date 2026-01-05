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

impl SessionRepository {
    /// Create a new repository with the given connection
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
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
