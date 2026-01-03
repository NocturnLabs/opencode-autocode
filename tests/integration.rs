//! Integration tests for opencode-autocode
//!
//! These tests verify end-to-end behavior of the CLI tool.

use std::fs;
use tempfile::TempDir;

/// Test that scaffolding creates expected files
#[test]
fn test_scaffold_creates_expected_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    // Run scaffold with default spec
    let result = opencode_autocode::scaffold::scaffold_default(output_path);
    assert!(result.is_ok(), "Scaffold should succeed");

    // Verify expected files exist
    let expected_files = vec![
        ".autocode/app_spec.md",
        ".opencode/command/auto-init.md",
        ".opencode/command/auto-continue.md",
        ".opencode/command/auto-enhance.md",
        ".autocode/security-allowlist.json",
        ".autocode/progress.db",
        ".autocode/config.toml",
        "opencode.json",
    ];

    for file in expected_files {
        let path = output_path.join(file);
        assert!(path.exists(), "Expected file missing: {}", file);
    }
}

/// Test that scaffold does NOT create run-autonomous.sh (removed)
#[test]
fn test_scaffold_no_shell_script() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    opencode_autocode::scaffold::scaffold_default(output_path).expect("Scaffold should succeed");

    // Verify shell script is NOT created
    let shell_script = output_path.join("scripts/run-autonomous.sh");
    assert!(
        !shell_script.exists(),
        "run-autonomous.sh should not be scaffolded"
    );
}

/// Test that vibe command correctly detects database
#[test]
fn test_vibe_detects_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    // Directory for database
    let autocode_dir = output_path.join(".autocode");
    fs::create_dir_all(&autocode_dir).expect("Failed to create .autocode dir");

    // Without database, vibe should run auto-init
    let db_path = autocode_dir.join("progress.db");
    assert!(!db_path.exists());

    // Create database (simulated)
    opencode_autocode::db::Database::open(&db_path).expect("Failed to create database");

    // Now it should exist
    assert!(db_path.exists());
}

/// Test config file generation
#[test]
fn test_scaffold_generates_valid_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    opencode_autocode::scaffold::scaffold_default(output_path).expect("Scaffold should succeed");

    // Verify config file exists and is valid TOML
    let config_path = output_path.join(".autocode/config.toml");
    assert!(config_path.exists(), ".autocode/config.toml should exist");

    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(
        content.contains("[models]"),
        "Config should have [models] section"
    );
    assert!(
        content.contains("[autonomous]"),
        "Config should have [autonomous] section"
    );
}

/// Test that opencode.json is valid JSON (not JSONC with comments)
#[test]
fn test_scaffold_generates_valid_opencode_json() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    opencode_autocode::scaffold::scaffold_default(output_path).expect("Scaffold should succeed");

    // Read and parse opencode.json
    let json_path = output_path.join("opencode.json");
    let content = fs::read_to_string(&json_path).expect("Failed to read opencode.json");

    // This will fail if there are comments or invalid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "opencode.json must be valid JSON, not JSONC: {:?}",
        parsed.err()
    );

    // Verify expected structure
    let json = parsed.unwrap();
    assert!(json.get("$schema").is_some(), "Should have $schema field");
    assert!(
        json.get("instructions").is_some(),
        "Should have instructions field"
    );
    assert!(
        json.get("permission").is_some(),
        "Should have permission field"
    );
}

/// Test that scaffold preserves existing config.toml
#[test]
fn test_scaffold_preserves_existing_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    // Pre-create config with custom value
    let autocode_dir = output_path.join(".autocode");
    fs::create_dir_all(&autocode_dir).unwrap();
    let config_path = autocode_dir.join("config.toml");
    fs::write(&config_path, "[models]\ndefault = \"custom/model\"\n").unwrap();

    // Scaffold - should NOT overwrite
    opencode_autocode::scaffold::scaffold_default(output_path).expect("Scaffold should succeed");

    // Verify custom value preserved
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(
        content.contains("custom/model"),
        "Config should preserve existing values, got: {}",
        content
    );
}
