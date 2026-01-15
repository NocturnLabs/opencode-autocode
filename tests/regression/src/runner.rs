use crate::config::RegressionConfig;
use crate::functional_tests;
use crate::integration_tests;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Result of a single regression test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Test category
    pub category: String,
    /// Whether the test passed
    pub passed: bool,
    /// Execution duration
    pub duration_ms: u64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Summary of regression test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total number of tests run
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed_tests: usize,
    /// Number of tests that failed
    pub failed_tests: usize,
    /// Total execution time
    pub total_duration_ms: u64,
    /// Test results
    pub results: Vec<TestResult>,
}

/// Regression test runner
pub struct RegressionRunner {
    config: RegressionConfig,
}

impl RegressionRunner {
    /// Create a new regression runner with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = RegressionConfig::default();
        Ok(Self { config })
    }

    /// Create a new regression runner with custom configuration
    pub fn with_config(config: RegressionConfig) -> Self {
        Self { config }
    }

    /// Load configuration from file
    pub fn from_config_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config = RegressionConfig::from_file(&path.to_path_buf())?;
        Ok(Self { config })
    }

    /// Run all regression tests
    pub async fn run_all_tests(&self) -> Result<TestSummary, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // Discover test cases
        let test_cases = self.discover_test_cases()?;

        // Run tests with concurrency control
        let results = self.run_tests_concurrent(test_cases).await;

        let total_duration = start_time.elapsed();

        let passed_tests = results.iter().filter(|r| r.passed).count();
        let failed_tests = results.len() - passed_tests;

        let summary = TestSummary {
            total_tests: results.len(),
            passed_tests,
            failed_tests,
            total_duration_ms: total_duration.as_millis() as u64,
            results,
        };

        // Generate reports
        self.generate_reports(&summary)?;

        Ok(summary)
    }

    /// Discover test cases from the test cases directory
    fn discover_test_cases(&self) -> Result<Vec<TestCase>, Box<dyn std::error::Error>> {
        let mut test_cases = Vec::new();

        // Walk through test cases directory
        for entry in walkdir::WalkDir::new(&self.config.test_cases_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let test_case: TestCase = serde_json::from_reader(std::fs::File::open(path)?)?;
                test_cases.push(test_case);
            }
        }

        Ok(test_cases)
    }

    /// Run tests with concurrency control
    async fn run_tests_concurrent(&self, test_cases: Vec<TestCase>) -> Vec<TestResult> {
        let mut results = Vec::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
            self.config.execution.max_concurrent_tests,
        ));

        let mut handles = Vec::new();

        for test_case in test_cases {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                let result = Self::run_single_test(&config, test_case).await;
                drop(permit);
                result
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        results
    }

    /// Run a single test case
    async fn run_single_test(config: &RegressionConfig, test_case: TestCase) -> TestResult {
        let start_time = Instant::now();

        let result = timeout(
            Duration::from_secs(config.execution.test_timeout_seconds),
            Self::execute_test_case(config, &test_case),
        )
        .await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(())) => TestResult {
                name: test_case.name,
                category: test_case.category,
                passed: true,
                duration_ms: duration.as_millis() as u64,
                error_message: None,
                metadata: HashMap::new(),
            },
            Ok(Err(e)) => TestResult {
                name: test_case.name,
                category: test_case.category,
                passed: false,
                duration_ms: duration.as_millis() as u64,
                error_message: Some(e.to_string()),
                metadata: HashMap::new(),
            },
            Err(_) => TestResult {
                name: test_case.name,
                category: test_case.category,
                passed: false,
                duration_ms: duration.as_millis() as u64,
                error_message: Some("Test timed out".to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Execute a test case based on its type
    async fn execute_test_case(
        config: &RegressionConfig,
        test_case: &TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match test_case.test_type.as_str() {
            // Functional tests
            "generator_prompt" => {
                functional_tests::test_generator_functionality(config, &test_case.config).await
            }
            "spec_validation" => {
                functional_tests::test_spec_validation(config, &test_case.config).await
            }
            "cli_execution" => {
                functional_tests::test_cli_execution(config, &test_case.config).await
            }
            "end_to_end" => {
                functional_tests::test_end_to_end_workflow(config, &test_case.config).await
            }
            "spec_sanitization" => {
                functional_tests::test_spec_sanitization(config, &test_case.config).await
            }
            // Integration tests
            "full_project_generation" => {
                integration_tests::test_full_project_generation(config, &test_case.config).await
            }
            "template_processing" => {
                integration_tests::test_template_processing(config, &test_case.config).await
            }
            "cli_command_sequence" => {
                integration_tests::test_cli_command_sequence(config, &test_case.config).await
            }
            "error_handling" => {
                integration_tests::test_error_handling(config, &test_case.config).await
            }
            _ => Err(format!("Unknown test type: {}", test_case.test_type).into()),
        }
    }

    /// Generate test reports
    fn generate_reports(&self, summary: &TestSummary) -> Result<(), Box<dyn std::error::Error>> {
        // Create results directory if it doesn't exist
        std::fs::create_dir_all(&self.config.results_dir)?;

        // Generate JSON report
        if self
            .config
            .reporting
            .output_formats
            .contains(&"json".to_string())
        {
            let json_path = self.config.results_dir.join("results.json");
            let json_content = serde_json::to_string_pretty(summary)?;
            std::fs::write(json_path, json_content)?;
        }

        // Generate summary text report
        if self
            .config
            .reporting
            .output_formats
            .contains(&"summary".to_string())
        {
            let summary_path = self.config.results_dir.join("summary.txt");
            let summary_content = format!(
                "Regression Test Summary\n\
                 =====================\n\
                 Total Tests: {}\n\
                 Passed: {}\n\
                 Failed: {}\n\
                 Success Rate: {:.1}%\n\
                 Total Duration: {:.2}s\n",
                summary.total_tests,
                summary.passed_tests,
                summary.failed_tests,
                if summary.total_tests > 0 {
                    (summary.passed_tests as f64 / summary.total_tests as f64) * 100.0
                } else {
                    0.0
                },
                summary.total_duration_ms as f64 / 1000.0
            );
            std::fs::write(summary_path, summary_content)?;
        }

        Ok(())
    }
}

/// Definition of a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name: String,
    /// Test category
    pub category: String,
    /// Test type
    pub test_type: String,
    /// Test configuration
    pub config: HashMap<String, serde_json::Value>,
}
