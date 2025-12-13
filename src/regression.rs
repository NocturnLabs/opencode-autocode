//! Regression testing module for feature_list.json validation
//!
//! This module provides functionality to parse feature_list.json and run
//! regression checks on features marked as passing.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Represents a single feature in feature_list.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Feature category (functional, style, integration, performance)
    pub category: String,

    /// Human-readable description of the feature
    pub description: String,

    /// Verification steps for manual testing
    pub steps: Vec<String>,

    /// Whether this feature currently passes all tests
    pub passes: bool,

    /// Optional shell command for automated verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_command: Option<String>,
}

/// Result of a single feature check
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub description: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub was_automated: bool,
}

/// Summary of regression check execution
#[derive(Debug, Clone)]
pub struct RegressionSummary {
    pub total_features: usize,
    pub passing_features: usize,
    pub automated_passed: usize,
    pub automated_failed: usize,
    pub manual_required: usize,
    pub results: Vec<CheckResult>,
}

/// Parse feature_list.json from the given path
pub fn parse_feature_list(path: &Path) -> Result<Vec<Feature>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read feature list: {}", path.display()))?;

    let features: Vec<Feature> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse feature list: {}", path.display()))?;

    Ok(features)
}

/// Run regression checks on all passing features
pub fn run_regression_check(
    path: &Path,
    category_filter: Option<&str>,
    verbose: bool,
) -> Result<RegressionSummary> {
    let features = parse_feature_list(path)?;

    let total_features = features.len();
    let passing_features: Vec<_> = features
        .iter()
        .filter(|f| f.passes)
        .filter(|f| {
            category_filter
                .map(|cat| f.category.eq_ignore_ascii_case(cat))
                .unwrap_or(true)
        })
        .collect();

    let mut automated_passed = 0;
    let mut automated_failed = 0;
    let mut manual_required = 0;
    let mut results = Vec::new();

    for feature in &passing_features {
        if verbose {
            println!("Checking: {}", feature.description);
        }

        if let Some(ref cmd) = feature.verification_command {
            // Run automated verification
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .with_context(|| format!("Failed to execute: {}", cmd))?;

            if output.status.success() {
                automated_passed += 1;
                results.push(CheckResult {
                    description: feature.description.clone(),
                    passed: true,
                    error_message: None,
                    was_automated: true,
                });

                if verbose {
                    println!("  ✓ PASS");
                }
            } else {
                automated_failed += 1;
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                results.push(CheckResult {
                    description: feature.description.clone(),
                    passed: false,
                    error_message: Some(stderr),
                    was_automated: true,
                });

                if verbose {
                    println!("  ✗ FAIL");
                }
            }
        } else {
            // No automated verification available
            manual_required += 1;
            results.push(CheckResult {
                description: feature.description.clone(),
                passed: true, // Assume manual-only features still pass
                error_message: None,
                was_automated: false,
            });

            if verbose {
                println!("  ○ MANUAL (no verification_command)");
            }
        }
    }

    Ok(RegressionSummary {
        total_features,
        passing_features: passing_features.len(),
        automated_passed,
        automated_failed,
        manual_required,
        results,
    })
}

/// Print a formatted report of the regression check results
pub fn report_results(summary: &RegressionSummary) {
    println!();
    println!("════════════════════════════════════════");
    println!("       REGRESSION CHECK SUMMARY");
    println!("════════════════════════════════════════");
    println!();
    println!(
        "Total features:        {}",
        summary.total_features
    );
    println!(
        "Features checked:      {}",
        summary.passing_features
    );
    println!();
    println!("Automated tests:");
    println!(
        "  ✓ Passed:           {}",
        summary.automated_passed
    );
    println!(
        "  ✗ Failed:           {}",
        summary.automated_failed
    );
    println!(
        "  ○ Manual required:  {}",
        summary.manual_required
    );
    println!();

    if summary.automated_failed > 0 {
        println!("❌ REGRESSIONS DETECTED:");
        for result in &summary.results {
            if result.was_automated && !result.passed {
                println!("  • {}", result.description);
                if let Some(ref err) = result.error_message {
                    // Print first line of error only
                    if let Some(first_line) = err.lines().next() {
                        println!("    └─ {}", first_line);
                    }
                }
            }
        }
        println!();
        println!("Action: Fix regressions before continuing.");
    } else {
        println!("✅ All automated regression tests passed!");
        if summary.manual_required > 0 {
            println!(
                "   Reminder: {} features require manual verification.",
                summary.manual_required
            );
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_feature_list() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"[
            {
                "category": "functional",
                "description": "Test feature 1",
                "steps": ["Step 1", "Step 2"],
                "passes": true,
                "verification_command": "echo test"
            },
            {
                "category": "functional",
                "description": "Test feature 2",
                "steps": ["Step 1"],
                "passes": false
            }
        ]"#;
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_feature_list() {
        let file = create_test_feature_list();
        let features = parse_feature_list(file.path()).unwrap();

        assert_eq!(features.len(), 2);
        assert_eq!(features[0].description, "Test feature 1");
        assert!(features[0].passes);
        assert!(features[0].verification_command.is_some());
    }

    #[test]
    fn test_run_regression_check() {
        let file = create_test_feature_list();
        let summary = run_regression_check(file.path(), None, false).unwrap();

        assert_eq!(summary.total_features, 2);
        assert_eq!(summary.passing_features, 1);
        assert_eq!(summary.automated_passed, 1);
        assert_eq!(summary.automated_failed, 0);
    }
}
