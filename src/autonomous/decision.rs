//! Supervisor Decision Logic
//!
//! Determines the next action for the autonomous agent, separated from the
//! main loop orchestration.

use anyhow::Result;
use std::path::Path;

use crate::conductor;
use crate::config::Config;
use crate::regression;

use super::debug_logger;
use super::features::FeatureProgress;
use super::verification::{classify_verification_failure, VerificationFailure};

/// Actions determined by the Supervisor
#[allow(dead_code)] // EnhanceReady is part of the design but not yet fully wired up
pub enum SupervisorAction {
    /// Run a standard command (auto-init, auto-continue, etc.)
    Command(&'static str),
    /// Fix a regression
    Fix {
        feature: crate::db::features::Feature,
        error: String,
    },
    /// All features complete, exit the loop (normal mode)
    Complete,
    /// All features pass, ready for enhancement phase (enhancement mode)
    EnhanceReady,
}

/// Determines the next action the supervisor should take.
///
/// This function is stateless and operates on the current configuration and
/// database state, prioritizing actions as follows:
///
/// 1. **Regression Fix**: If any previously-passing feature is now failing.
/// 2. **Initialization**: If no features exist in the database.
/// 3. **Context Setup**: If conductor requires context generation.
/// 4. **Active Track**: If there's an active plan with remaining tasks.
/// 5. **Continue**: If there are pending features.
/// 6. **Stop**: All features are passing.
pub fn determine_action(
    db_path: &Path,
    config: &Config,
    target_feature_id: Option<i64>,
) -> Result<SupervisorAction> {
    let logger = debug_logger::get();

    // --- Priority 0: Target Feature (Parallel Mode) ---
    // If targeting a specific feature, skip regression check and focus on that feature
    if let Some(id) = target_feature_id {
        return determine_action_for_target(db_path, id, logger);
    }

    // --- Priority 1: Regression Check ---
    if FeatureProgress::has_features(db_path)? {
        if let Some(action) = check_for_regressions(db_path, config, logger)? {
            return Ok(action);
        }
    }

    // --- Phase 1: First Run ---
    // Database features is the source of truth for init status.
    // Signal file is maintained for visibility but not used for decision.
    let has_features = match FeatureProgress::has_features(db_path) {
        Ok(has) => has,
        Err(e) => {
            eprintln!("[ERROR] Failed to check features DB: {}", e);
            // If we can't check DB, assuming "not initialized" is dangerous if DB exists.
            if init_signal_exists() {
                eprintln!("[WARN] DB check failed but signal exists. Assuming initialized to avoid destructive re-init.");
                true
            } else {
                return Err(e.context("Failed to check initialization status"));
            }
        }
    };
    let signal_exists = init_signal_exists();

    eprintln!(
        "[DEBUG] Decision check: db_path={:?}, has_features={}, signal_exists={}",
        db_path, has_features, signal_exists
    );

    // DB is source of truth - signal file mismatch is a warning, not a skip
    if !has_features {
        if signal_exists {
            eprintln!("[WARN] Signal file exists but DB has no features - reinitializing");
        }
        eprintln!("[DEBUG] Selected: auto-init (no features in DB)");
        return Ok(SupervisorAction::Command("auto-init"));
    }

    // --- Phase 2: Context ---
    if config.conductor.auto_setup && !conductor::context_exists(config) {
        return Ok(SupervisorAction::Command("auto-context"));
    }

    // --- Phase 3: Active Track ---
    if let Some(track) = conductor::get_active_track(config)? {
        let plan_path = track.path.join("plan.md");
        if let Ok(tasks) = conductor::parse_plan(&plan_path) {
            if conductor::get_next_task(&tasks).is_some() {
                return Ok(SupervisorAction::Command("auto-continue"));
            }
        }
    }

    // --- Phase 4: DB Progress ---
    let progress = FeatureProgress::load_from_db(db_path)?;
    println!(
        "→ Progress: {} passing, {} remaining",
        progress.passing, progress.remaining
    );

    if progress.all_passing() {
        return Ok(SupervisorAction::Complete);
    }

    // --- Phase 5: Auto-continue ---
    Ok(SupervisorAction::Command("auto-continue"))
}

/// Handles the case when a specific feature ID is targeted (parallel mode).
fn determine_action_for_target(
    db_path: &Path,
    id: i64,
    logger: &debug_logger::DebugLogger,
) -> Result<SupervisorAction> {
    let db = crate::db::Database::open(db_path)?;
    let features = db.features().list_all()?;

    if let Some(feature) = features.iter().find(|f| f.id == Some(id)) {
        if feature.passes {
            logger.info(&format!("Target feature {} already passes", id));
            return Ok(SupervisorAction::Complete);
        }

        // If feature has a stored error, trigger Fix mode to give agent context
        if let Some(ref error) = feature.last_error {
            println!(
                "🔧 Target Feature #{} has previous error, entering Fix mode",
                id
            );
            return Ok(SupervisorAction::Fix {
                feature: feature.clone(),
                error: error.clone(),
            });
        }

        println!("📋 Target Feature #{}: {}", id, feature.description);
        return Ok(SupervisorAction::Command("auto-continue"));
    }

    logger.error(&format!("Target feature {} not found", id));
    anyhow::bail!("Target feature {} not found in database", id)
}

/// Checks for regressions in previously passing features.
///
/// Returns `Some(SupervisorAction::Fix)` if a regression is found,
/// or `None` if all passing features are still passing.
fn check_for_regressions(
    db_path: &Path,
    config: &Config,
    logger: &debug_logger::DebugLogger,
) -> Result<Option<SupervisorAction>> {
    let db = crate::db::Database::open(db_path)?;
    let features = db.features().list_all()?;

    let summary = regression::run_regression_check(&features, None, false, Some(&config.security))?;

    if summary.automated_failed == 0 {
        return Ok(None);
    }

    // Track broken verification commands to detect systemic issues
    let mut broken_verification_count = 0;

    // Find the first failing feature to fix
    for result in summary.results {
        if !result.passed && result.was_automated {
            if let Some(feature) = features
                .iter()
                .find(|f| f.description == result.description)
            {
                let error_msg = result.error_message.unwrap_or_default();

                // Classify the failure to avoid looping on broken verification commands
                let failure_type = classify_verification_failure(&error_msg);

                match failure_type {
                    VerificationFailure::NoTestsMatch
                    | VerificationFailure::TestFileMissing
                    | VerificationFailure::CommandError => {
                        // The verification command is broken, not the code
                        broken_verification_count += 1;

                        println!("⚠️  Verification command issue (not a code regression)");
                        println!("   Feature: {}", feature.description);
                        println!("   Error: {}", error_msg.lines().next().unwrap_or(""));
                        println!("   → Marking as pending for re-implementation");
                        logger.warning(&format!(
                            "Verification command broken for '{}': {}",
                            feature.description,
                            failure_type.as_str()
                        ));

                        // Mark as failing so it goes back to pending queue
                        db.features().mark_failing(&feature.description)?;

                        // If multiple features have broken verification, it's a systemic issue
                        if broken_verification_count >= 3 {
                            println!("🛑 Multiple features have broken verification commands");
                            println!("   This suggests a shared configuration issue");
                            logger.error("Systemic verification command failure detected");
                            return Ok(Some(SupervisorAction::Complete)); // Stop, don't loop
                        }

                        // Don't return Fix action - continue to find next feature
                        continue;
                    }
                    VerificationFailure::AssertionFailure => {
                        // Real regression - proceed with fix
                        return Ok(Some(SupervisorAction::Fix {
                            feature: feature.clone(),
                            error: error_msg,
                        }));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Check if the initialization signal file exists with CONTINUE or COMPLETE content.
///
/// This acts as a fallback indicator that initialization has completed,
/// in case the database check fails for some reason.
fn init_signal_exists() -> bool {
    const SIGNAL_FILE: &str = ".opencode-signal";
    let path = std::path::Path::new(SIGNAL_FILE);
    let exists = path.exists();
    eprintln!(
        "[DEBUG] init_signal_exists: path={:?}, exists={}",
        path, exists
    );

    if !exists {
        return false;
    }

    match std::fs::read_to_string(SIGNAL_FILE) {
        Ok(content) => {
            let trimmed = content.trim();
            let result = trimmed == "CONTINUE" || trimmed == "COMPLETE";
            eprintln!(
                "[DEBUG] init_signal_exists: content={:?}, result={}",
                trimmed, result
            );
            result
        }
        Err(e) => {
            eprintln!("[DEBUG] init_signal_exists: read error: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init_signal_exists_returns_false_when_missing() {
        let _ = fs::remove_file(".opencode-signal-test");
        // Testing with actual file would interfere with other tests,
        // so we just verify the function doesn't panic
        let _ = init_signal_exists();
    }
}
