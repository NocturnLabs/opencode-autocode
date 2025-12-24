//! Feature list utilities for the autonomous loop

use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

use crate::regression;

/// Feature progress status
pub struct FeatureProgress {
    pub passing: usize,
    pub remaining: usize,
}

impl FeatureProgress {
    /// Count passing and remaining features from feature_list.json
    pub fn load(path: &Path) -> Result<Self> {
        let features = regression::parse_feature_list(path)?;
        let passing = features.iter().filter(|f| f.passes).count();
        let remaining = features.iter().filter(|f| !f.passes).count();
        Ok(Self { passing, remaining })
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

/// Get descriptions of currently passing features
pub fn get_passing_feature_descriptions(path: &Path) -> Result<HashSet<String>> {
    if !path.exists() {
        return Ok(HashSet::new());
    }

    let features = regression::parse_feature_list(path)?;
    Ok(features
        .into_iter()
        .filter(|f| f.passes)
        .map(|f| f.description)
        .collect())
}

/// Detect newly completed features by comparing before/after sets
pub fn detect_newly_completed(before: &HashSet<String>, after: &HashSet<String>) -> Vec<String> {
    after.difference(before).cloned().collect()
}
