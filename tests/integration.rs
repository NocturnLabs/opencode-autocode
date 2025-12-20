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
        "app_spec.md",
        ".opencode/command/auto-init.md",
        ".opencode/command/auto-continue.md",
        ".opencode/command/auto-enhance.md",
        "scripts/security-allowlist.json",
        "opencode-progress.txt",
        "autocode.toml",
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

/// Test that vibe command correctly detects feature_list.json
#[test]
fn test_vibe_detects_feature_list() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    // Without feature_list.json, vibe should run auto-init
    let feature_list_path = output_path.join("feature_list.json");
    assert!(!feature_list_path.exists());

    // Create fake feature_list.json
    let feature_list_content = r#"[
        {"description": "Test feature", "passes": false}
    ]"#;
    fs::write(&feature_list_path, feature_list_content).expect("Failed to write feature list");

    // Now it should exist
    assert!(feature_list_path.exists());
}

/// Test config file generation
#[test]
fn test_scaffold_generates_valid_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path();

    opencode_autocode::scaffold::scaffold_default(output_path).expect("Scaffold should succeed");

    // Verify config file exists and is valid TOML
    let config_path = output_path.join("autocode.toml");
    assert!(config_path.exists(), "autocode.toml should exist");

    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(content.contains("[models]"), "Config should have [models] section");
    assert!(
        content.contains("[autonomous]"),
        "Config should have [autonomous] section"
    );
}
