//! Feature list utilities for the autonomous loop

use anyhow::Result;

use std::path::Path;

use crate::db;

/// Feature progress status
pub struct FeatureProgress {
    pub passing: usize,
    pub remaining: usize,
}

impl FeatureProgress {
    /// Count passing and remaining features from SQLite database
    pub fn load_from_db(db_path: &Path) -> Result<Self> {
        let database = db::Database::open(db_path)?;
        let repo = database.features();
        let (passing, remaining) = repo.count()?;
        Ok(Self { passing, remaining })
    }

    /// Check if the database has any features (determines if init has run)
    pub fn has_features(db_path: &Path) -> bool {
        if !db_path.exists() {
            return false;
        }
        if let Ok(database) = db::Database::open(db_path) {
            if let Ok((passing, remaining)) = database.features().count() {
                return passing + remaining > 0;
            }
        }
        false
    }

    /// Total number of features
    pub fn total(&self) -> usize {
        self.passing + self.remaining
    }

    /// Check if all features are passing
    pub fn all_passing(&self) -> bool {
        self.remaining == 0 && self.passing > 0
    }
}

/// Get descriptions of currently passing features from database

/// Get the first pending (not passing) feature from the database
pub fn get_first_pending_feature(db_path: &Path) -> Result<Option<db::features::Feature>> {
    if !db_path.exists() {
        return Ok(None);
    }

    let database = db::Database::open(db_path)?;
    let features = database.features().list_all()?;

    // Find first feature where passes = false (i.e., passes = 0)
    Ok(features.into_iter().find(|f| !f.passes))
}

/// Get up to N pending (not passing) features for parallel processing
pub fn get_pending_features(db_path: &Path, limit: usize) -> Result<Vec<db::features::Feature>> {
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let database = db::Database::open(db_path)?;
    let features = database.features().list_all()?;

    // Get first N non-passing features
    Ok(features
        .into_iter()
        .filter(|f| !f.passes)
        .take(limit)
        .collect())
}

/// Get a specific feature by its ID
pub fn get_feature_by_id(db_path: &Path, id: i64) -> Result<Option<db::features::Feature>> {
    if !db_path.exists() {
        return Ok(None);
    }

    let database = db::Database::open(db_path)?;
    let features = database.features().list_all()?;

    Ok(features.into_iter().find(|f| f.id == Some(id)))
}
