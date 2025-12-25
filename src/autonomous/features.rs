//! Feature list utilities for the autonomous loop

use anyhow::Result;
use std::collections::HashSet;
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
pub fn get_passing_feature_descriptions(db_path: &Path) -> Result<HashSet<String>> {
    if !db_path.exists() {
        return Ok(HashSet::new());
    }

    let database = db::Database::open(db_path)?;
    let descriptions = database.features().get_passing_descriptions()?;
    Ok(descriptions)
}

/// Detect newly completed features by comparing before/after sets
pub fn detect_newly_completed(before: &HashSet<String>, after: &HashSet<String>) -> Vec<String> {
    after.difference(before).cloned().collect()
}
