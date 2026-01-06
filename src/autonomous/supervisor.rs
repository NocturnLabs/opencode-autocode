use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::conductor;
use crate::config::Config;
use crate::regression;

use super::debug_logger;
use super::display;
use super::features::{self, FeatureProgress};
use super::git;
use super::security;
use super::session;
use super::settings::{handle_session_result, LoopAction, LoopSettings};
use super::templates;
use super::verification::{classify_verification_failure, VerificationFailure};
use super::webhook;

/// Actions determined by the Supervisor
pub enum SupervisorAction {
    /// Run a standard command (auto-init, auto-continue, etc.)
    Command(&'static str),
    /// Fix a regression
    Fix {
        feature: crate::db::features::Feature,
        error: String,
    },
    /// Stop the loop (completed or otherwise)
    Stop,
}

pub fn determine_action(
    db_path: &Path,
    config: &Config,
    target_feature_id: Option<i64>,
) -> Result<SupervisorAction> {
    let logger = debug_logger::get();

    // If targeting a specific feature (parallel mode), skip regression check and focus on that feature
    if let Some(id) = target_feature_id {
        let db = crate::db::Database::open(db_path)?;
        let features = db.features().list_all()?;
        if let Some(feature) = features.iter().find(|f| f.id == Some(id)) {
            if feature.passes {
                logger.info(&format!("Target feature {} already passes", id));
                return Ok(SupervisorAction::Stop);
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
        } else {
            logger.error(&format!("Target feature {} not found", id));
            return Ok(SupervisorAction::Stop);
        }
    }

    // 0. REGRESSION CHECK (Priority #1)
    if FeatureProgress::has_features(db_path) {
        let db = crate::db::Database::open(db_path)?;
        let features = db.features().list_all()?;

        // Run check on ALL features (not just passing ones to be safe, but regression checks usually imply passing ones)
        // actually regression check only checks passing ones.
        // We want to verify that previously passing features are STILL passing.
        let summary =
            regression::run_regression_check(&features, None, false, Some(&config.security))?;

        if summary.automated_failed > 0 {
            // Find the first failing feature to fix
            for result in summary.results {
                if !result.passed && result.was_automated {
                    if let Some(feature) = features
                        .iter()
                        .find(|f| f.description == result.description)
                    {
                        let error_msg = result.error_message.unwrap_or_default();

                        // SMART STUCK DETECTION: Classify the failure
                        let failure_type = classify_verification_failure(&error_msg);

                        match failure_type {
                            VerificationFailure::NoTestsMatch
                            | VerificationFailure::TestFileMissing
                            | VerificationFailure::CommandError => {
                                // The verification command is broken, not the code
                                // Mark as failing and move on instead of looping
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

                                // Don't return Fix action - continue to find next feature
                                continue;
                            }
                            VerificationFailure::AssertionFailure => {
                                // Real regression - proceed with fix
                                return Ok(SupervisorAction::Fix {
                                    feature: feature.clone(),
                                    error: error_msg,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Phase 1: First run
    if !FeatureProgress::has_features(db_path) {
        return Ok(SupervisorAction::Command("auto-init"));
    }

    // Phase 2: Context
    if config.conductor.auto_setup && !conductor::context_exists(config) {
        return Ok(SupervisorAction::Command("auto-context"));
    }

    // Phase 3: Active Track
    if let Some(track) = conductor::get_active_track(config)? {
        let plan_path = track.path.join("plan.md");
        if let Ok(tasks) = conductor::parse_plan(&plan_path) {
            if conductor::get_next_task(&tasks).is_some() {
                return Ok(SupervisorAction::Command("auto-continue"));
            }
        }
    }

    // Phase 4: DB Progress
    let progress = FeatureProgress::load_from_db(db_path)?;
    println!(
        "→ Progress: {} passing, {} remaining",
        progress.passing, progress.remaining
    );

    if progress.all_passing() {
        return Ok(SupervisorAction::Stop);
    }

    // Phase 5: Auto-continue
    Ok(SupervisorAction::Command("auto-continue"))
}

pub fn run_supervisor_loop(
    config: &Config,
    settings: &LoopSettings,
    enhancement_mode: bool,
    target_feature_id: Option<i64>,
) -> Result<()> {
    let db_path = Path::new(&settings.database_file);
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;
    let mut last_run_success = true;
    let logger = debug_logger::get();

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            logger.info("Reached max iterations");
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        // Check for stop signal BEFORE printing session header (avoids "ghost sessions")
        if session::stop_signal_exists() {
            logger.info("Supervisor: Stop signal received.");
            break;
        }

        // 1. Determine Action (Supervisor Logic) — check this before printing header
        let action = determine_action(db_path, config, target_feature_id)?;

        // State for verification after session
        let mut active_feature = None;

        // Exit early if all features are complete (don't print a ghost session)
        if matches!(action, SupervisorAction::Stop) && !enhancement_mode {
            logger.info("Supervisor: All features complete.");
            break;
        }

        // Now safe to print the session header
        logger.separator();
        logger.info(&format!("Session {} starting", iteration));
        display::display_session_header(iteration);

        let command_name = match action {
            SupervisorAction::Stop => {
                // enhancement_mode is true here (non-enhancement Stop is handled above)
                match templates::handle_enhancement_phase(db_path, config, settings, iteration) {
                    Ok(LoopAction::Continue) => {
                        iteration += 1;
                        "auto-enhance-active".to_string()
                    }
                    _ => {
                        logger.info("Supervisor: Stop signal received or enhancement exited.");
                        break;
                    }
                }
            }
            SupervisorAction::Command(cmd) => {
                logger.info(&format!("Supervisor: Selected command '{}'", cmd));

                // For auto-continue, inject feature context (supervisor controls what LLM works on)
                if cmd == "auto-continue" {
                    let feature_opt = if let Some(id) = target_feature_id {
                        features::get_feature_by_id(db_path, id)?
                    } else {
                        features::get_first_pending_feature(db_path)?
                    };

                    if let Some(feature) = feature_opt {
                        active_feature = Some(feature.clone());
                        templates::generate_continue_template(&feature)?;
                        println!(
                            "📋 Feature #{}: {}",
                            feature.id.unwrap_or(0),
                            feature.description
                        );
                        "auto-continue-active".to_string()
                    } else {
                        // No pending features, use standard continue
                        cmd.to_string()
                    }
                } else {
                    cmd.to_string()
                }
            }
            SupervisorAction::Fix { feature, error } => {
                logger.info(&format!(
                    "Supervisor: REGRESSION DETECTED in '{}'",
                    feature.description
                ));
                println!("🚨 REGRESSION DETECTED: {}", feature.description);
                println!("→ Switching to auto-fix mode...");

                active_feature = Some(feature.clone());

                // Generate dynamic auto-fix template
                templates::generate_fix_template(&feature, &error, db_path)?;
                "auto-fix-active".to_string()
            }
        };

        println!("→ Running: opencode run --command /{}", command_name);
        println!();

        // 3. Run Session
        let result = session::execute_opencode_session(
            &command_name,
            &settings.model,
            &settings.log_level,
            None,
            settings.session_timeout,
            logger,
        )?;

        // 4. Supervisor Verification (only if session didn't crash)
        // Check result ref to avoid move (needed for handle_session_result later)
        let session_ok = matches!(&result, session::SessionResult::Continue);

        if session_ok {
            if let Some(feature) = active_feature {
                println!("🔍 Supervisor: Verifying feature...");
                println!("   Feature: {}", feature.description);

                if let Some(cmd) = &feature.verification_command {
                    // Use security-validated command runner
                    let output = match security::run_verified_command(cmd, &config.security, None) {
                        Ok(out) => out,
                        Err(e) => {
                            println!("  🚫 Security: Command blocked");
                            println!("     {}", e);
                            last_run_success = false;
                            let db = crate::db::Database::open(db_path)?;
                            db.features().mark_failing_with_error(
                                &feature.description,
                                Some(&format!("Security blocked: {}", e)),
                            )?;
                            continue;
                        }
                    };

                    if output.status.success() {
                        println!("  ✅ Verification PASSED!");
                        last_run_success = true;

                        // Mark as passing
                        let db = crate::db::Database::open(db_path)?;
                        db.features().mark_passing(&feature.description)?;
                        println!("  ✓ Marked as passing in DB");
                        logger.info(&format!(
                            "Verification PASSED for '{}'",
                            feature.description
                        ));

                        // NEW: Mark in Conductor plan if active track matches
                        if let Some(track) = conductor::get_active_track(config)? {
                            let plan_path = track.path.join("plan.md");
                            if let Ok(tasks) = conductor::parse_plan(&plan_path) {
                                if let Some(task) = conductor::get_next_task(&tasks) {
                                    // Only mark if the description matches or it's the next logical task
                                    let _ =
                                        conductor::mark_task_complete(&plan_path, task.line_number);
                                    println!(
                                        "  ✓ Marked task complete in plan.md: {}",
                                        task.description
                                    );
                                }
                            }
                        }

                        // Commit if needed
                        if settings.auto_commit {
                            match git::commit_completed_feature(
                                &feature.description,
                                settings.verbose,
                            ) {
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
                            &feature,
                            iteration,
                            progress.passing,
                            progress.total(),
                        );
                    } else {
                        println!("  ❌ Verification FAILED");
                        println!("     Command: {}", cmd);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let error_msg = if !stderr.is_empty() {
                            println!("     Error: {}", stderr.lines().next().unwrap_or(""));
                            stderr.to_string()
                        } else if !stdout.is_empty() {
                            // Some test frameworks output to stdout
                            stdout.to_string()
                        } else {
                            "Verification command failed with no output".to_string()
                        };

                        // STASH PROTOCOL: Capture diff to verify what failed, then clean up
                        println!("  → Stashing failed attempt to capture context...");
                        let stash_msg = format!("autocode-failure-{}", feature.description);
                        let mut diff_context = String::new();

                        // 1. Try to stash
                        match git::stash_push(&stash_msg) {
                            Ok(true) => {
                                // 2. If stashed, get the diff
                                if let Ok(diff) = git::stash_show_latest() {
                                    if !diff.is_empty() {
                                        // Limit diff size to avoid huge context
                                        let truncated_diff = if diff.len() > 10000 {
                                            format!("{}\n... (truncated)", &diff[..10000])
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
                        let final_error_msg = format!("{}{}", error_msg, diff_context);

                        // Mark as failing with error context for auto-fix
                        let db = crate::db::Database::open(db_path)?;
                        db.features().mark_failing_with_error(
                            &feature.description,
                            Some(&final_error_msg),
                        )?;

                        println!("  → Feature marked as failing (will auto-fix next iteration with failure diff)");
                        logger.info(&format!(
                            "Verification FAILED for '{}'",
                            feature.description
                        ));

                        last_run_success = false;
                    }
                } else {
                    println!("  ❌ No verification command (manual check required)");
                    last_run_success = false;
                    // If we don't have a command, we mark it as failing to avoid infinite loop
                    let db = crate::db::Database::open(db_path)?;
                    db.features().mark_failing_with_error(
                        &feature.description,
                        Some("No verification command produced by agent"),
                    )?;
                }
            }
        }

        // Display token usage
        if let Some(ref stats) = session::fetch_token_stats() {
            display::display_token_stats(stats);
        }

        // Handle loop continuation
        match handle_session_result(result, settings, &mut consecutive_errors) {
            LoopAction::Continue => {
                thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
            }
            LoopAction::Break => break,
            LoopAction::RetryWithBackoff(backoff) => {
                thread::sleep(Duration::from_secs(backoff as u64));
            }
        }
    }

    if last_run_success {
        Ok(())
    } else {
        anyhow::bail!("Autonomous run complete but the last feature failed verification.")
    }
}
