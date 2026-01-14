//! Supervisor Decision Logic
//!
//! Determines the next action for the autonomous agent, separated from the
//! main loop orchestration.

use anyhow::Result;
use std::path::Path;

use crate::conductor;
use crate::config::Config;
use crate::regression;

use super::features::FeatureProgress;
use super::verification::{classify_verification_failure, VerificationFailure};
use crate::common::logging as debug_logger;

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

    // --- Phase 1: Initialization ---
    // Check both DB features AND the explicit initialization marker.
    // If we have either, we are initialized.
    let (is_initialized, has_features) = match crate::db::Database::open(db_path) {
        Ok(db) => {
            let marker = db.meta().is_initialized().unwrap_or(false);
            let features = FeatureProgress::has_features(db_path).unwrap_or(false);
            (marker, features)
        }
        Err(_) => (false, false),
    };

    let project_root = db_path
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let signal = read_init_signal(&project_root);

    eprintln!(
        "[DEBUG] Decision check: db_path={:?}, is_initialized={}, has_features={}, signal={:?}",
        db_path, is_initialized, has_features, signal
    );

    // Initialized if marker exists OR we have features
    let actually_initialized = is_initialized || has_features;

    if !actually_initialized {
        if let Some(ref value) = signal {
            if value.eq_ignore_ascii_case("continue") {
                println!(
                    "[WARN] Signal file exists but project not marked initialized - trusting signal once"
                );
                println!("[DEBUG] Selected: auto-continue (signal exists)");
                clear_init_signal(&project_root);
                return Ok(SupervisorAction::Command("auto-continue"));
            }
            if value.eq_ignore_ascii_case("complete") {
                println!(
                    "[WARN] Complete signal present but project not initialized - rerunning init"
                );
                clear_init_signal(&project_root);
            }
        }
        println!("[DEBUG] Selected: auto-init (project not initialized)");
        return Ok(SupervisorAction::Command("auto-init"));
    }

    if signal.is_some() {
        clear_init_signal(&project_root);
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

    let sample_size = if config.agent.verification_sample_size == 0 {
        None
    } else {
        Some(config.agent.verification_sample_size as usize)
    };
    let summary = regression::run_regression_check(
        &features,
        None,
        sample_size,
        false,
        Some(&config.security),
    )?;

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

/// Read the initialization signal file, if present.
fn read_init_signal(project_root: &Path) -> Option<String> {
    const SIGNAL_FILE: &str = ".opencode-signal";
    let path = project_root.join(SIGNAL_FILE);
    let exists = path.exists();
    eprintln!(
        "[DEBUG] read_init_signal: path={:?}, exists={}",
        path, exists
    );

    if !exists {
        return None;
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(e) => {
            eprintln!("[DEBUG] read_init_signal: read error: {}", e);
            None
        }
    }
}

/// Clear the initialization signal file after consuming it.
fn clear_init_signal(project_root: &Path) {
    let path = project_root.join(".opencode-signal");
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_init_signal_returns_none_when_missing() {
        let _ = fs::remove_file(".opencode-signal");
        let result = read_init_signal(Path::new("."));
        assert!(result.is_none());
    }
}
