//! Feature repository for database operations
//!
//! Provides CRUD operations for features, replacing feature_list.json.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Feature data structure (matches the old JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Database ID (optional for JSON import)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    /// Feature category (functional, style, integration, performance)
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Verification steps
    pub steps: Vec<String>,

    /// Whether this feature passes all tests
    pub passes: bool,

    /// Optional shell command for automated verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_command: Option<String>,

    /// Last verification error (for auto-fix context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

/// Repository for feature CRUD operations
pub struct FeatureRepository {
    conn: Arc<Mutex<Connection>>,
}

impl FeatureRepository {
    /// Create a new repository with the given connection
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a new feature
    pub fn insert(&self, feature: &Feature) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO features (category, description, passes, verification_command)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                feature.category,
                feature.description,
                feature.passes as i32,
                feature.verification_command,
            ],
        )
        .context("Failed to insert feature")?;

        let feature_id = conn.last_insert_rowid();

        // Insert steps
        for (order, step) in feature.steps.iter().enumerate() {
            conn.execute(
                "INSERT INTO feature_steps (feature_id, step_order, step_text)
                 VALUES (?1, ?2, ?3)",
                params![feature_id, order as i32, step],
            )
            .context("Failed to insert feature step")?;
        }

        Ok(feature_id)
    }

    /// Get all features
    pub fn list_all(&self) -> Result<Vec<Feature>> {
        let conn = self.conn.lock().unwrap();
        self.query_features(&conn, "SELECT * FROM features ORDER BY id")
    }

    pub fn count(&self) -> Result<(usize, usize)> {
        let conn = self.conn.lock().unwrap();

        let passing: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM features WHERE passes = 1",
                [],
                |row| row.get(0),
            )
            .context("Failed to count passing features")?;

        let remaining: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM features WHERE passes = 0",
                [],
                |row| row.get(0),
            )
            .context("Failed to count remaining features")?;

        Ok((passing as usize, remaining as usize))
    }

    /// Mark a feature as passing by description (clears last_error)
    pub fn mark_passing(&self, description: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows = conn
            .execute(
                "UPDATE features SET passes = 1, last_error = NULL WHERE description = ?1",
                params![description],
            )
            .context("Failed to mark feature as passing")?;

        Ok(rows > 0)
    }

    /// Mark a feature as failing by description with an optional error message
    pub fn mark_failing(&self, description: &str) -> Result<bool> {
        self.mark_failing_with_error(description, None)
    }

    /// Mark a feature as failing with an error message for auto-fix context
    pub fn mark_failing_with_error(&self, description: &str, error: Option<&str>) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows = conn
            .execute(
                "UPDATE features SET passes = 0, last_error = ?2 WHERE description = ?1",
                params![description, error],
            )
            .context("Failed to mark feature as failing")?;

        Ok(rows > 0)
    }

    /// Import features from a JSON file (one-time migration)
    pub fn import_from_json(&self, path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let features: Vec<Feature> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;

        let mut count = 0;
        for feature in features {
            // Skip if already exists (by description)
            if self.exists_by_description(&feature.description)? {
                continue;
            }
            self.insert(&feature)?;
            count += 1;
        }

        Ok(count)
    }

    /// Export features to JSON format
    pub fn export_to_json(&self, path: &Path) -> Result<()> {
        let features = self.list_all()?;
        let content = serde_json::to_string_pretty(&features)?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Check if a feature exists by description
    pub fn exists_by_description(&self, description: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM features WHERE description = ?1",
                params![description],
                |row| row.get(0),
            )
            .context("Failed to check feature existence")?;

        Ok(count > 0)
    }

    /// Helper to query features and load their steps
    fn query_features(&self, conn: &Connection, sql: &str) -> Result<Vec<Feature>> {
        let mut stmt = conn.prepare(sql).context("Failed to prepare query")?;

        let feature_iter = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,            // id
                    row.get::<_, String>(1)?,         // category
                    row.get::<_, String>(2)?,         // description
                    row.get::<_, i32>(3)? != 0,       // passes
                    row.get::<_, Option<String>>(4)?, // verification_command
                    row.get::<_, Option<String>>(5)?, // last_error
                ))
            })
            .context("Failed to query features")?;

        let mut features = Vec::new();

        // Collect feature data first to release stmt borrow
        let mut feature_data = Vec::new();
        for feature in feature_iter {
            feature_data.push(feature?);
        }

        // Now load steps for each feature
        for (id, category, description, passes, verification_command, last_error) in feature_data {
            // Load steps for this feature
            let mut step_stmt = conn
                .prepare(
                    "SELECT step_text FROM feature_steps WHERE feature_id = ?1 ORDER BY step_order",
                )
                .context("Failed to prepare steps query")?;

            let steps = step_stmt
                .query_map(params![id], |row| row.get::<_, String>(0))
                .context("Failed to query steps")?
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to read steps")?;

            features.push(Feature {
                id: Some(id),
                category,
                description,
                steps,
                passes,
                verification_command,
                last_error,
            });
        }

        Ok(features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils::tests::setup_test_db;

    #[test]
    fn test_insert_and_list() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        let feature = Feature {
            id: None,
            category: "functional".to_string(),
            description: "Test feature".to_string(),
            steps: vec!["Step 1".to_string(), "Step 2".to_string()],
            passes: false,
            verification_command: Some("echo test".to_string()),
            last_error: None,
        };

        let id = repo.insert(&feature).unwrap();
        assert!(id > 0);

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 0);
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_mark_passing() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        let feature = Feature {
            id: None,
            category: "functional".to_string(),
            description: "Test feature".to_string(),
            steps: vec![],
            passes: false,
            verification_command: None,
            last_error: None,
        };

        repo.insert(&feature).unwrap();

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 0);
        assert_eq!(remaining, 1);

        repo.mark_passing("Test feature").unwrap();

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 1);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_count() {
        let (_temp, db) = setup_test_db();
        let repo = db.features();

        // Insert some features
        for i in 0..5 {
            let feature = Feature {
                id: None,
                category: "functional".to_string(),
                description: format!("Feature {}", i),
                steps: vec![],
                passes: i % 2 == 0, // 0, 2, 4 pass; 1, 3 fail
                verification_command: None,
                last_error: None,
            };
            repo.insert(&feature).unwrap();
        }

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 3);
        assert_eq!(remaining, 2);
    }
}
