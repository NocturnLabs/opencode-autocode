//! Regression Testing Suite for opencode-forger
//!
//! This module provides comprehensive regression testing capabilities
//! to ensure that changes to the codebase don't break existing functionality.

pub mod automation;
pub mod config;
pub mod functional_tests;
pub mod integration_tests;
pub mod runner;

use crate::runner::{RegressionRunner, TestSummary};

/// Run regression tests with default configuration
pub async fn run_regression_tests() -> Result<TestSummary, Box<dyn std::error::Error>> {
    let runner = RegressionRunner::new()?;
    runner.run_all_tests().await
}

/// Run regression tests with custom configuration file
pub async fn run_regression_tests_with_config(
    config_path: &std::path::Path,
) -> Result<TestSummary, Box<dyn std::error::Error>> {
    let runner = RegressionRunner::from_config_file(config_path)?;
    runner.run_all_tests().await
}
