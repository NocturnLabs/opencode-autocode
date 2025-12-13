use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the regression testing suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Base directory for regression tests
    pub base_dir: PathBuf,
    /// Directory containing test fixtures
    pub fixtures_dir: PathBuf,
    /// Directory containing baseline results
    pub baselines_dir: PathBuf,
    /// Directory containing test case definitions
    pub test_cases_dir: PathBuf,
    /// Directory for storing test results
    pub results_dir: PathBuf,
    /// Test execution settings
    pub execution: ExecutionConfig,
    /// Reporting settings
    pub reporting: ReportingConfig,
}

/// Test execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Timeout for individual tests (in seconds)
    pub test_timeout_seconds: u64,
    /// Maximum number of concurrent tests
    pub max_concurrent_tests: usize,
    /// Whether to stop on first failure
    pub fail_fast: bool,
    /// Environment variables to set during testing
    pub environment: std::collections::HashMap<String, String>,
}

/// Test reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Whether to generate detailed reports
    pub detailed_reports: bool,
    /// Whether to compare results with baselines
    pub baseline_comparison: bool,
    /// Report output formats
    pub output_formats: Vec<String>,
    /// Whether to save results for future baseline comparison
    pub save_results: bool,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("tests/regression"),
            fixtures_dir: PathBuf::from("tests/regression/fixtures"),
            baselines_dir: PathBuf::from("tests/regression/baselines"),
            test_cases_dir: PathBuf::from("tests/regression/test_cases"),
            results_dir: PathBuf::from("tests/regression/results"),
            execution: ExecutionConfig::default(),
            reporting: ReportingConfig::default(),
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            test_timeout_seconds: 300, // 5 minutes
            max_concurrent_tests: 4,
            fail_fast: false,
            environment: std::collections::HashMap::new(),
        }
    }
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            detailed_reports: true,
            baseline_comparison: true,
            output_formats: vec!["json".to_string(), "html".to_string()],
            save_results: true,
        }
    }
}

impl RegressionConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: RegressionConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }
}
