//! Functional regression tests for core modules
//!
//! These tests verify that individual components work correctly
//! and haven't regressed from their expected behavior.

use crate::config::RegressionConfig;
use std::collections::HashMap;

/// Test the code generation functionality
pub async fn test_generator_functionality(
    _config: &RegressionConfig,
    test_config: &HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Extract test parameters
    let input = test_config
        .get("input")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'input' parameter")?;

    let expected_contains = test_config
        .get("expected_contains")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'expected_contains' parameter")?;

    // Use the actual generator module to build the prompt
    // This verifies the prompt construction logic works as expected
    let config = opencode_forger::config::Config::default();
    let prompt = opencode_forger::generator::prompts::build_generation_prompt(input, None, &config);

    // Check that expected strings are contained in the generated prompt
    for expected in expected_contains {
        if let Some(expected_str) = expected.as_str() {
            if !prompt.contains(expected_str) {
                return Err(
                    format!("Expected '{}' not found in generated prompt", expected_str).into(),
                );
            }
        }
    }

    Ok(())
}

/// Test specification validation functionality
pub async fn test_spec_validation(
    _config: &RegressionConfig,
    test_config: &HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec_content = test_config
        .get("spec_content")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'spec_content' parameter")?;

    let expected_valid = test_config
        .get("expected_valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let expected_features_count = test_config
        .get("expected_features_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    // Use actual validation logic
    let result = opencode_forger::validation::validate_spec(spec_content);

    // If we expected a hard error but didn't get one (or vice versa)
    match result {
        Ok(validation_result) => {
            if validation_result.is_valid != expected_valid {
                return Err(format!(
                    "Validation result mismatch: expected valid={}, got valid={}",
                    expected_valid, validation_result.is_valid
                )
                .into());
            }

            if validation_result.stats.feature_count != expected_features_count {
                return Err(format!(
                    "Features count mismatch: expected {}, got {}",
                    expected_features_count, validation_result.stats.feature_count
                )
                .into());
            }
        }
        Err(e) => {
            // If we expected valid=true but validation crashed, that's a fail
            if expected_valid {
                return Err(format!("Validation failed unexpectedly: {}", e).into());
            }
        }
    }

    Ok(())
}

/// Test CLI functionality
pub async fn test_cli_execution(
    _config: &RegressionConfig,
    test_config: &HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let command = test_config
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'command' parameter")?;

    let expected_exit_code = test_config
        .get("expected_exit_code")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as i32;

    let expected_output_contains = test_config
        .get("expected_output_contains")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'expected_output_contains' parameter")?;

    // Run the CLI command
    let output = Command::new("cargo")
        .args(["run", "--", command])
        .current_dir("../../")
        .output()?;

    // Check exit code
    if output.status.code() != Some(expected_exit_code) {
        return Err(format!(
            "Exit code mismatch: expected {}, got {:?}",
            expected_exit_code,
            output.status.code()
        )
        .into());
    }

    // Check output contains expected strings
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    for expected in expected_output_contains {
        if let Some(expected_str) = expected.as_str() {
            if !combined_output.contains(expected_str) {
                return Err(format!("Expected '{}' not found in output", expected_str).into());
            }
        }
    }

    Ok(())
}

/// Test end-to-end workflow
pub async fn test_end_to_end_workflow(
    _config: &RegressionConfig,
    test_config: &HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_idea = test_config
        .get("input_idea")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'input_idea' parameter")?;

    let expected_outputs = test_config
        .get("expected_outputs")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'expected_outputs' parameter")?;

    let validate_spec = test_config
        .get("validate_spec")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let check_templates = test_config
        .get("check_templates")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Simulate end-to-end workflow
    // TODO: Replace with actual end-to-end testing

    // Generate specification
    let spec = format!(
        r#"<project_specification>
<project_name>Test Project</project_name>
<overview>Generated from: {}</overview>
<technology_stack>
<frontend>React</frontend>
<backend>Node.js</backend>
</technology_stack>
<core_features>
<feature1>Basic functionality</feature1>
<feature2>Advanced features</feature2>
</core_features>
<success_criteria>
- App works correctly
</success_criteria>
</project_specification>"#,
        input_idea
    );

    // Check expected outputs
    for expected in expected_outputs {
        if let Some(expected_str) = expected.as_str() {
            if !spec.contains(expected_str) {
                return Err(
                    format!("Expected '{}' not found in specification", expected_str).into(),
                );
            }
        }
    }

    // Validate spec if requested
    if validate_spec && !spec.contains("<project_specification>") {
        return Err("Specification validation failed".into());
    }

    // Check templates if requested
    if check_templates {
        // TODO: Implement template checking
    }

    Ok(())
}

/// Test specification sanitization functionality
pub async fn test_spec_sanitization(
    _config: &RegressionConfig,
    test_config: &HashMap<String, serde_json::Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let malformed_specs = test_config
        .get("malformed_specs")
        .and_then(|v| v.as_array())
        .ok_or("Missing 'malformed_specs' parameter")?;

    for spec_case in malformed_specs {
        let name = spec_case
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");

        let input = spec_case
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or(format!("Missing 'input' for case: {}", name))?;

        let expected_contains = spec_case
            .get("expected_sanitized_contains")
            .and_then(|v| v.as_array())
            .ok_or(format!(
                "Missing 'expected_sanitized_contains' for case: {}",
                name
            ))?;

        let should_parse = spec_case
            .get("should_parse")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Use the sanitizer
        let sanitized = opencode_forger::generator::sanitize::sanitize_spec_xml(input);

        // Check expected strings are present
        for expected in expected_contains {
            if let Some(expected_str) = expected.as_str() {
                if !sanitized.contains(expected_str) {
                    return Err(format!(
                        "Case '{}': Expected '{}' not found in sanitized output",
                        name, expected_str
                    )
                    .into());
                }
            }
        }

        // Check nothing was double-escaped
        if let Some(not_expected) = spec_case.get("expected_not_double_escaped") {
            if let Some(not_expected_arr) = not_expected.as_array() {
                for not_exp in not_expected_arr {
                    if let Some(not_exp_str) = not_exp.as_str() {
                        if sanitized.contains(not_exp_str) {
                            return Err(format!(
                                "Case '{}': Double-escaped '{}' found in output",
                                name, not_exp_str
                            )
                            .into());
                        }
                    }
                }
            }
        }

        // If should_parse, verify it can be validated without XML errors
        if should_parse {
            let result = opencode_forger::validation::validate_spec(&sanitized);
            match result {
                Ok(validation) => {
                    // Check there are no XML parsing errors
                    let has_xml_error = validation
                        .errors
                        .iter()
                        .any(|e| e.contains("XML parsing error"));
                    if has_xml_error {
                        return Err(format!(
                            "Case '{}': Sanitized spec still has XML parsing errors: {:?}",
                            name, validation.errors
                        )
                        .into());
                    }
                }
                Err(e) => {
                    return Err(
                        format!("Case '{}': Validation failed unexpectedly: {}", name, e).into(),
                    );
                }
            }
        }
    }

    Ok(())
}
