//! Test automation and reporting utilities
//!
//! This module provides utilities for automated test execution,
//! result comparison with baselines, and comprehensive reporting.

use crate::config::RegressionConfig;
use crate::runner::{TestResult, TestSummary};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Baseline data for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBaseline {
    pub test_name: String,
    pub expected_result: ExpectedTestResult,
    pub performance_baseline: PerformanceBaseline,
    pub last_updated: String,
}

/// Expected test result for baseline comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedTestResult {
    pub should_pass: bool,
    pub expected_duration_ms: Option<u64>,
    pub expected_error_pattern: Option<String>,
}

/// Performance baseline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub max_duration_ms: u64,
    pub avg_duration_ms: u64,
    pub min_duration_ms: u64,
}

/// Test automation runner with baseline comparison
pub struct AutomatedTestRunner {
    config: RegressionConfig,
    baselines: HashMap<String, TestBaseline>,
}

impl AutomatedTestRunner {
    /// Create a new automated test runner
    pub fn new(config: RegressionConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let baselines = Self::load_baselines(&config.baselines_dir)?;
        Ok(Self { config, baselines })
    }

    /// Run tests with automation and baseline comparison
    pub async fn run_automated_tests(&self) -> Result<AutomatedTestSummary, Box<dyn std::error::Error>> {
        // Run the basic test suite
        let summary = self.run_basic_tests().await?;

        // Compare with baselines
        let baseline_comparison = self.compare_with_baselines(&summary)?;

        // Generate automated reports
        let reports = self.generate_automated_reports(&summary, &baseline_comparison)?;

        Ok(AutomatedTestSummary {
            basic_summary: summary,
            baseline_comparison,
            reports,
        })
    }

    /// Run basic tests (delegates to RegressionRunner)
    async fn run_basic_tests(&self) -> Result<TestSummary, Box<dyn std::error::Error>> {
        use crate::runner::RegressionRunner;
        let runner = RegressionRunner::with_config(self.config.clone());
        runner.run_all_tests().await
    }

    /// Compare test results with baselines
    fn compare_with_baselines(&self, summary: &TestSummary) -> Result<BaselineComparison, Box<dyn std::error::Error>> {
        let mut regressions = Vec::new();
        let mut improvements = Vec::new();
        let mut new_tests = Vec::new();

        for result in &summary.results {
            if let Some(baseline) = self.baselines.get(&result.name) {
                // Compare with baseline
                let comparison = self.compare_single_result(result, baseline);
                match comparison {
                    TestComparison::Regression(details) => regressions.push(details),
                    TestComparison::Improvement(details) => improvements.push(details),
                    TestComparison::NoChange => {} // Expected
                }
            } else {
                // New test without baseline
                new_tests.push(result.name.clone());
            }
        }

        Ok(BaselineComparison {
            regressions,
            improvements,
            new_tests,
            total_comparisons: summary.results.len(),
        })
    }

    /// Compare a single test result with its baseline
    fn compare_single_result(&self, result: &TestResult, baseline: &TestBaseline) -> TestComparison {
        // Check if pass/fail status changed
        if result.passed != baseline.expected_result.should_pass {
            return TestComparison::Regression(RegressionDetails {
                test_name: result.name.clone(),
                issue_type: "status_change".to_string(),
                description: format!(
                    "Test status changed from {} to {}",
                    if baseline.expected_result.should_pass { "pass" } else { "fail" },
                    if result.passed { "pass" } else { "fail" }
                ),
                severity: if result.passed { "improvement" } else { "regression" }.to_string(),
            });
        }

        // Check performance regression
        if let Some(expected_duration) = baseline.expected_result.expected_duration_ms {
            let performance_factor = result.duration_ms as f64 / expected_duration as f64;
            if performance_factor > 2.0 { // More than 2x slower
                return TestComparison::Regression(RegressionDetails {
                    test_name: result.name.clone(),
                    issue_type: "performance".to_string(),
                    description: format!(
                        "Performance regression: {}ms vs expected {}ms ({:.1}x slower)",
                        result.duration_ms, expected_duration, performance_factor
                    ),
                    severity: "performance_regression".to_string(),
                });
            } else if performance_factor < 0.5 { // More than 2x faster
                return TestComparison::Improvement(ImprovementDetails {
                    test_name: result.name.clone(),
                    improvement_type: "performance".to_string(),
                    description: format!(
                        "Performance improvement: {}ms vs expected {}ms ({:.1}x faster)",
                        result.duration_ms, expected_duration, 1.0 / performance_factor
                    ),
                });
            }
        }

        TestComparison::NoChange
    }

    /// Generate automated reports
    fn generate_automated_reports(
        &self,
        summary: &TestSummary,
        comparison: &BaselineComparison,
    ) -> Result<AutomatedReports, Box<dyn std::error::Error>> {
        // Create results directory
        fs::create_dir_all(&self.config.results_dir)?;

        // Generate summary report
        let summary_report = self.generate_summary_report(summary, comparison)?;

        // Generate detailed report
        let detailed_report = self.generate_detailed_report(summary, comparison)?;

        // Generate baseline update recommendations
        let baseline_updates = self.generate_baseline_updates(summary)?;

        Ok(AutomatedReports {
            summary_report,
            detailed_report,
            baseline_updates,
        })
    }

    /// Generate summary report
    fn generate_summary_report(
        &self,
        summary: &TestSummary,
        comparison: &BaselineComparison,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut report = String::new();
        report.push_str("# Regression Test Summary Report\n\n");
        report.push_str(&format!("**Execution Time:** {:.2}s\n", summary.total_duration_ms as f64 / 1000.0));
        report.push_str(&format!("**Tests Run:** {}\n", summary.total_tests));
        report.push_str(&format!("**Passed:** {} ({:.1}%)\n",
            summary.passed_tests,
            (summary.passed_tests as f64 / summary.total_tests as f64) * 100.0));
        report.push_str(&format!("**Failed:** {}\n", summary.failed_tests));
        report.push_str(&format!("**Regressions:** {}\n", comparison.regressions.len()));
        report.push_str(&format!("**Improvements:** {}\n", comparison.improvements.len()));
        report.push_str(&format!("**New Tests:** {}\n", comparison.new_tests.len()));

        if !comparison.regressions.is_empty() {
            report.push_str("\n## 🚨 Regressions Detected\n\n");
            for regression in &comparison.regressions {
                report.push_str(&format!("- **{}**: {}\n", regression.test_name, regression.description));
            }
        }

        Ok(report)
    }

    /// Generate detailed report
    fn generate_detailed_report(
        &self,
        summary: &TestSummary,
        comparison: &BaselineComparison,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut report = String::new();
        report.push_str("# Detailed Regression Test Report\n\n");

        // Test results table
        report.push_str("## Test Results\n\n");
        report.push_str("| Test Name | Status | Duration | Category |\n");
        report.push_str("|-----------|--------|----------|----------|\n");

        for result in &summary.results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            report.push_str(&format!("| {} | {} | {}ms | {} |\n",
                result.name, status, result.duration_ms, result.category));
        }

        // Baseline comparison
        if !comparison.regressions.is_empty() || !comparison.improvements.is_empty() {
            report.push_str("\n## Baseline Comparison\n\n");

            if !comparison.regressions.is_empty() {
                report.push_str("### Regressions\n\n");
                for regression in &comparison.regressions {
                    report.push_str(&format!("- **{}** ({}) - {}\n",
                        regression.test_name, regression.severity, regression.description));
                }
            }

            if !comparison.improvements.is_empty() {
                report.push_str("\n### Improvements\n\n");
                for improvement in &comparison.improvements {
                    report.push_str(&format!("- **{}** - {}\n",
                        improvement.test_name, improvement.description));
                }
            }
        }

        Ok(report)
    }

    /// Generate baseline update recommendations
    fn generate_baseline_updates(&self, summary: &TestSummary) -> Result<String, Box<dyn std::error::Error>> {
        let mut updates = String::new();
        updates.push_str("# Baseline Update Recommendations\n\n");

        for result in &summary.results {
            if result.passed {
                updates.push_str(&format!("- Update baseline for **{}**: duration {}ms\n",
                    result.name, result.duration_ms));
            }
        }

        Ok(updates)
    }

    /// Load baseline data from files
    fn load_baselines(baselines_dir: &Path) -> Result<HashMap<String, TestBaseline>, Box<dyn std::error::Error>> {
        let mut baselines = HashMap::new();

        if !baselines_dir.exists() {
            return Ok(baselines);
        }

        for entry in fs::read_dir(baselines_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let baseline: TestBaseline = serde_json::from_reader(fs::File::open(&path)?)?;
                baselines.insert(baseline.test_name.clone(), baseline);
            }
        }

        Ok(baselines)
    }
}

/// Automated test summary with baseline comparison
#[derive(Debug, Clone)]
pub struct AutomatedTestSummary {
    pub basic_summary: TestSummary,
    pub baseline_comparison: BaselineComparison,
    pub reports: AutomatedReports,
}

/// Baseline comparison results
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    pub regressions: Vec<RegressionDetails>,
    pub improvements: Vec<ImprovementDetails>,
    pub new_tests: Vec<String>,
    pub total_comparisons: usize,
}

/// Details of a regression
#[derive(Debug, Clone)]
pub struct RegressionDetails {
    pub test_name: String,
    pub issue_type: String,
    pub description: String,
    pub severity: String,
}

/// Details of an improvement
#[derive(Debug, Clone)]
pub struct ImprovementDetails {
    pub test_name: String,
    pub improvement_type: String,
    pub description: String,
}

/// Generated reports
#[derive(Debug, Clone)]
pub struct AutomatedReports {
    pub summary_report: String,
    pub detailed_report: String,
    pub baseline_updates: String,
}

/// Test comparison result
enum TestComparison {
    Regression(RegressionDetails),
    Improvement(ImprovementDetails),
    NoChange,
}