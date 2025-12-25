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

    /// Get all passing features
    pub fn list_passing(&self) -> Result<Vec<Feature>> {
        let conn = self.conn.lock().unwrap();
        self.query_features(&conn, "SELECT * FROM features WHERE passes = 1 ORDER BY id")
    }

    /// Get all remaining (non-passing) features
    pub fn list_remaining(&self) -> Result<Vec<Feature>> {
        let conn = self.conn.lock().unwrap();
        self.query_features(&conn, "SELECT * FROM features WHERE passes = 0 ORDER BY id")
    }

    /// Count passing and remaining features
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

    /// Mark a feature as passing by description
    pub fn mark_passing(&self, description: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows = conn
            .execute(
                "UPDATE features SET passes = 1 WHERE description = ?1",
                params![description],
            )
            .context("Failed to mark feature as passing")?;

        Ok(rows > 0)
    }

    /// Mark a feature as failing by description
    pub fn mark_failing(&self, description: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows = conn
            .execute(
                "UPDATE features SET passes = 0 WHERE description = ?1",
                params![description],
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

    /// Get passing feature descriptions as a set
    pub fn get_passing_descriptions(&self) -> Result<std::collections::HashSet<String>> {
        let features = self.list_passing()?;
        Ok(features.into_iter().map(|f| f.description).collect())
    }

    /// Helper to query features and load their steps
    fn query_features(&self, conn: &Connection, sql: &str) -> Result<Vec<Feature>> {
        let mut stmt = conn.prepare(sql).context("Failed to prepare query")?;

        let feature_rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,            // id
                    row.get::<_, String>(1)?,         // category
                    row.get::<_, String>(2)?,         // description
                    row.get::<_, i32>(3)? != 0,       // passes
                    row.get::<_, Option<String>>(4)?, // verification_command
                ))
            })
            .context("Failed to query features")?;

        let mut features = Vec::new();

        for row in feature_rows {
            let (id, category, description, passes, verification_command) =
                row.context("Failed to read feature row")?;

            // Load steps for this feature
            let steps = self.load_steps(conn, id)?;

            features.push(Feature {
                id: Some(id),
                category,
                description,
                steps,
                passes,
                verification_command,
            });
        }

        Ok(features)
    }

    /// Load steps for a feature
    fn load_steps(&self, conn: &Connection, feature_id: i64) -> Result<Vec<String>> {
        let mut stmt = conn
            .prepare(
                "SELECT step_text FROM feature_steps WHERE feature_id = ?1 ORDER BY step_order",
            )
            .context("Failed to prepare steps query")?;

        let steps = stmt
            .query_map(params![feature_id], |row| row.get::<_, String>(0))
            .context("Failed to query steps")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read steps")?;

        Ok(steps)
    }
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
        };

        let id = repo.insert(&feature).unwrap();
        assert!(id > 0);

        let features = repo.list_all().unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].description, "Test feature");
        assert_eq!(features[0].steps.len(), 2);
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
            };
            repo.insert(&feature).unwrap();
        }

        let (passing, remaining) = repo.count().unwrap();
        assert_eq!(passing, 3);
        assert_eq!(remaining, 2);
    }
}
