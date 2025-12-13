use opencode_regression_tests::{automation::AutomatedTestRunner, config::RegressionConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 OpenCode Regression Test Suite");
    println!("=================================");

    // Load configuration
    let config_path = std::env::var("REGRESSION_CONFIG_FILE")
        .map(|p| Path::new(&p).to_path_buf())
        .unwrap_or_else(|_| Path::new("config/default.toml").to_path_buf());

    println!("Using config file: {:?}", config_path);

    // Load config and create automated runner
    let config_content = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|_| include_str!("../config/default.toml").to_string());
    let config: RegressionConfig = toml::from_str(&config_content)?;

    let runner = AutomatedTestRunner::new(config)?;

    println!("Running automated regression tests...");
    let summary = runner.run_automated_tests().await?;

    let basic_summary = &summary.basic_summary;
    let comparison = &summary.baseline_comparison;

    println!("\n📊 Automated Test Results Summary");
    println!("=================================");
    println!("Total Tests: {}", basic_summary.total_tests);
    println!("Passed: {}", basic_summary.passed_tests);
    println!("Failed: {}", basic_summary.failed_tests);
    println!("Success Rate: {:.1}%",
             if basic_summary.total_tests > 0 {
                 (basic_summary.passed_tests as f64 / basic_summary.total_tests as f64) * 100.0
             } else {
                 0.0
             });
    println!("Total Duration: {:.2}s", basic_summary.total_duration_ms as f64 / 1000.0);
    println!("Regressions: {}", comparison.regressions.len());
    println!("Improvements: {}", comparison.improvements.len());
    println!("New Tests: {}", comparison.new_tests.len());

    if !comparison.regressions.is_empty() {
        println!("\n🚨 Regressions Detected:");
        for regression in &comparison.regressions {
            println!("  - {}: {}", regression.test_name, regression.description);
        }
    }

    if basic_summary.failed_tests > 0 {
        println!("\n❌ Failed Tests:");
        for result in &basic_summary.results {
            if !result.passed {
                println!("  - {}: {}", result.name, result.error_message.as_deref().unwrap_or("Unknown error"));
            }
        }
        std::process::exit(1);
    } else if comparison.regressions.is_empty() {
        println!("\n✅ All tests passed with no regressions!");
    } else {
        println!("\n⚠️  Tests passed but regressions detected!");
        std::process::exit(1);
    }

    // Save reports to files
    println!("\n📄 Saving reports...");
    std::fs::write("results/summary_report.md", &summary.reports.summary_report)?;
    std::fs::write("results/detailed_report.md", &summary.reports.detailed_report)?;
    std::fs::write("results/baseline_updates.md", &summary.reports.baseline_updates)?;
    println!("Reports saved to results/ directory");

    Ok(())
}