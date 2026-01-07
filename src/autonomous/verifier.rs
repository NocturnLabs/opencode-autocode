//! Feature Verification
//!
//! Handles running verification commands for features and classifying results.

use anyhow::Result;
use std::path::Path;
use std::process::Output;

use crate::conductor;
use crate::config::{Config, SecurityConfig};
use crate::db::features::Feature;

use super::debug_logger;
use super::features::FeatureProgress;
use super::git;
use super::security;
use super::settings::LoopSettings;
use super::webhook;

/// Result of verifying a feature
pub enum VerificationResult {
    /// Verification passed
    Passed,
    /// Verification failed with an error message and optional diff context
    Failed { error_message: String },
    /// No verification command was provided
    NoCommand,
    /// Command was blocked by security policy
    SecurityBlocked { reason: String },
}

/// Runs the verification command for a feature and returns the result.
///
/// This function:
/// 1. Validates the command against security policy.
/// 2. Executes the command.
/// 3. Classifies the output as pass/fail.
pub fn run_verification(
    feature: &Feature,
    security_config: &SecurityConfig,
) -> Result<VerificationResult> {
    let Some(ref cmd) = feature.verification_command else {
        return Ok(VerificationResult::NoCommand);
    };

    // Use security-validated command runner
    let output = match security::run_verified_command(cmd, security_config, None) {
        Ok(out) => out,
        Err(e) => {
            return Ok(VerificationResult::SecurityBlocked {
                reason: e.to_string(),
            });
        }
    };

    if output.status.success() {
        Ok(VerificationResult::Passed)
    } else {
        let error_message = extract_error_message(&output);
        Ok(VerificationResult::Failed { error_message })
    }
}

/// Extracts the error message from command output (preferring stderr).
fn extract_error_message(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stderr.is_empty() {
        stderr.to_string()
    } else if !stdout.is_empty() {
        stdout.to_string()
    } else {
        "Verification command failed with no output".to_string()
    }
}

/// Handles a successful verification by:
/// 1. Marking the feature as passing in the database.
/// 2. Marking the task complete in the conductor plan (if applicable).
/// 3. Auto-committing changes (if enabled).
/// 4. Notifying via webhook.
pub fn handle_verification_success(
    feature: &Feature,
    db_path: &Path,
    config: &Config,
    settings: &LoopSettings,
    iteration: usize,
) -> Result<()> {
    let logger = debug_logger::get();

    println!("  ✅ Verification PASSED!");

    // Mark as passing in the database
    let db = crate::db::Database::open(db_path)?;
    db.features().mark_passing(&feature.description)?;
    println!("  ✓ Marked as passing in DB");
    logger.info(&format!(
        "Verification PASSED for '{}'",
        feature.description
    ));

    // Mark in Conductor plan if active track matches
    if let Some(track) = conductor::get_active_track(config)? {
        let plan_path = track.path.join("plan.md");
        if let Ok(tasks) = conductor::parse_plan(&plan_path) {
            if let Some(task) = conductor::get_next_task(&tasks) {
                let _ = conductor::mark_task_complete(&plan_path, task.line_number);
                println!("  ✓ Marked task complete in plan.md: {}", task.description);
            }
        }
    }

    // Commit if needed
    if settings.auto_commit {
        match git::commit_completed_feature(&feature.description, settings.verbose) {
            Ok(_) => {
                logger.info(&format!(
                    "Auto-committed changes for '{}'",
                    feature.description
                ));
            }
            Err(e) => {
                logger.error(&format!(
                    "Failed to commit changes for '{}': {}",
                    feature.description, e
                ));
            }
        }
    }

    // Notify webhook
    let progress = FeatureProgress::load_from_db(db_path)?;
    let _ = webhook::notify_feature_complete(
        config,
        feature,
        iteration,
        progress.passing,
        progress.total(),
    );

    Ok(())
}

/// Handles a failed verification by:
/// 1. Stashing the failed attempt to capture context.
/// 2. Marking the feature as failing with error context.
pub fn handle_verification_failure(
    feature: &Feature,
    error_message: &str,
    db_path: &Path,
    settings: &LoopSettings,
) -> Result<()> {
    let logger = debug_logger::get();

    println!("  ❌ Verification FAILED");
    println!(
        "     Command: {}",
        feature.verification_command.as_deref().unwrap_or("N/A")
    );
    println!("     Error: {}", error_message.lines().next().unwrap_or(""));

    // STASH PROTOCOL: Capture diff to verify what failed, then clean up
    println!("  → Stashing failed attempt to capture context...");
    let stash_msg = format!("forger-failure-{}", feature.description);
    let mut diff_context = String::new();

    // 1. Try to stash
    match git::stash_push(&stash_msg) {
        Ok(true) => {
            // 2. If stashed, get the diff
            if let Ok(diff) = git::stash_show_latest() {
                if !diff.is_empty() {
                    // Limit diff size to avoid huge context
                    let truncated_diff = if diff.len() > 10000 {
                        format!("{}\\n... (truncated)", &diff[..10000])
                    } else {
                        diff
                    };
                    diff_context = format!(
                        "\n\n### Failed Implementation Diff:\n```diff\n{}\n```",
                        truncated_diff
                    );
                }
            }
            // 3. Drop the stash (we have the diff string, and we want clean slate)
            let _ = git::stash_drop();
            println!("  ✓ Stashed and captured diff for context");
        }
        Ok(false) => {
            // Nothing to stash (no changes made or empty)
            let _ = git::discard_changes(settings.verbose);
        }
        Err(e) => {
            logger.error(&format!("Stash failed: {}", e));
            let _ = git::discard_changes(settings.verbose);
        }
    }

    // Update the error message with the diff context
    let final_error_msg = format!("{}{}", error_message, diff_context);

    // Mark as failing with error context for auto-fix
    let db = crate::db::Database::open(db_path)?;
    db.features()
        .mark_failing_with_error(&feature.description, Some(&final_error_msg))?;

    println!("  → Feature marked as failing (will auto-fix next iteration with failure diff)");
    logger.info(&format!(
        "Verification FAILED for '{}'",
        feature.description
    ));

    Ok(())
}
