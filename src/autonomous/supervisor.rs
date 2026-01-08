//! Supervisor Loop
//!
//! The main orchestrator for the autonomous agent. Coordinates:
//! - Decision making (what action to take next)
//! - Session execution (running OpenCode)
//! - Verification (checking if features pass)
//!
//! This module focuses on loop orchestration; decision logic is in `decision.rs`
//! and verification logic is in `verifier.rs`.

use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::config::Config;

use super::debug_logger;
use super::decision::{determine_action, SupervisorAction};
use super::display;
use super::features;
use super::session;
use super::settings::{handle_session_result, LoopAction, LoopSettings};
use super::stats;
use super::templates;
use super::verifier::{
    handle_verification_failure, handle_verification_success, run_verification, VerificationResult,
};

/// Runs the main supervisor loop.
///
/// The loop is bounded by `settings.max_iterations` and can be stopped by a
/// stop signal file. Each iteration:
///
/// 1. Checks for stop signal or max iterations.
/// 2. Determines the next action (init, continue, fix, stop).
/// 3. Executes an OpenCode session.
/// 4. Verifies the result (if applicable).
/// 5. Handles the session result (continue, break, retry with backoff).
///
/// # NASA Power of 10 Compliance
/// - Loop has explicit upper bound (`max_iterations`).
/// - No recursion in control flow.
/// - All `Result` values are checked and handled.
pub fn run_supervisor_loop(
    config: &Config,
    settings: &LoopSettings,
    enhancement_mode: bool,
    target_feature_id: Option<i64>,
) -> Result<()> {
    let db_path = Path::new(&settings.database_file);
    let logger = debug_logger::get();

    // Mutable state for the loop
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;
    let mut no_progress_count = 0u32;
    let mut last_run_success = true;

    // --- Main Loop (Bounded by max_iterations) ---
    loop {
        iteration += 1;

        // --- Exit Condition 1: Max Iterations ---
        if iteration > settings.max_iterations {
            logger.info("Reached max iterations");
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        // --- Exit Condition 2: Stop Signal ---
        if session::stop_signal_exists() {
            logger.info("Supervisor: Stop signal received.");
            break;
        }

        // --- Step 1: Determine Action ---
        let action = determine_action(db_path, config, target_feature_id)?;

        // State for verification after session
        let mut active_feature = None;

        // Exit early if all features are complete (don't print a ghost session)
        if matches!(action, SupervisorAction::Complete) && !enhancement_mode {
            logger.info("Supervisor: All features complete.");
            break;
        }

        // Now safe to print the session header
        logger.separator();
        logger.info(&format!("Session {} starting", iteration));
        display::display_session_header(iteration);

        // --- Step 2: Prepare Command ---
        let command_name = match action {
            SupervisorAction::Complete => {
                // In normal mode, this is handled by early exit above
                // In enhancement_mode, route to enhancement phase
                if enhancement_mode {
                    match templates::handle_enhancement_phase(db_path, config, settings, iteration)
                    {
                        Ok(LoopAction::Continue) => {
                            iteration += 1;
                            "auto-enhance-active".to_string()
                        }
                        _ => {
                            logger.info("Supervisor: Enhancement phase exited.");
                            break;
                        }
                    }
                } else {
                    logger.info("Supervisor: All features complete.");
                    break;
                }
            }
            SupervisorAction::EnhanceReady => {
                // Same as Complete for now, enhancement mode handles this
                match templates::handle_enhancement_phase(db_path, config, settings, iteration) {
                    Ok(LoopAction::Continue) => {
                        iteration += 1;
                        "auto-enhance-active".to_string()
                    }
                    _ => {
                        logger.info("Supervisor: Enhancement phase exited.");
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
                        templates::generate_continue_template(
                            &feature,
                            settings.dual_model_enabled,
                        )?;
                        println!(
                            "📋 Feature #{}: {}",
                            feature.id.unwrap_or(0),
                            feature.description
                        );
                        "auto-continue-active".to_string()
                    } else {
                        // No pending features - skip this session rather than running blind
                        logger.warning("No pending feature found for auto-continue");
                        println!("⚠️ No pending feature found, skipping session");
                        no_progress_count += 1;
                        continue;
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
                templates::generate_fix_template(
                    &feature,
                    &error,
                    db_path,
                    settings.dual_model_enabled,
                )?;
                "auto-fix-active".to_string()
            }
        };

        println!("→ Running: opencode run --command /{}", command_name);
        println!();

        // --- Step 3: Execute Session ---
        let result = session::execute_opencode_session(
            session::SessionOptions {
                command: command_name.to_string(),
                model: settings.model.clone(),
                log_level: settings.log_level.clone(),
                session_id: None,
                timeout_minutes: settings.session_timeout,
                idle_timeout_seconds: settings.idle_timeout,
            },
            logger,
        )?;

        // --- Step 4: Verification ---
        // Both Continue and EarlyTerminated should trigger verification
        let session_ok = matches!(
            &result,
            session::SessionResult::Continue | session::SessionResult::EarlyTerminated { .. }
        );

        // Log warning for early termination (pattern-based, may be false positive)
        if let session::SessionResult::EarlyTerminated { ref trigger } = result {
            println!("⚠️ Session terminated via pattern match: {}", trigger);
            logger.warning(&format!("Early termination trigger: {}", trigger));
        }

        // Track whether this iteration made progress
        let mut made_progress = false;

        if session_ok {
            if let Some(ref feature) = active_feature {
                println!("🔍 Supervisor: Verifying feature...");
                println!("   Feature: {}", feature.description);

                let verification_result = run_verification(feature, &config.security)?;

                match verification_result {
                    VerificationResult::Passed => {
                        last_run_success = true;
                        made_progress = true;
                        handle_verification_success(feature, db_path, config, settings, iteration)?;
                    }
                    VerificationResult::Failed { error_message } => {
                        last_run_success = false;
                        handle_verification_failure(feature, &error_message, db_path, settings)?;
                    }
                    VerificationResult::NoCommand => {
                        println!("  ❌ No verification command (manual check required)");
                        last_run_success = false;
                        let db = crate::db::Database::open(db_path)?;
                        db.features().mark_failing_with_error(
                            &feature.description,
                            Some("No verification command produced by agent"),
                        )?;
                    }
                    VerificationResult::SecurityBlocked { reason } => {
                        println!("  🚫 Security: Command blocked");
                        println!("     {}", reason);
                        last_run_success = false;
                        let db = crate::db::Database::open(db_path)?;
                        db.features().mark_failing_with_error(
                            &feature.description,
                            Some(&format!("Security blocked: {}", reason)),
                        )?;
                        continue;
                    }
                }
            }
        }

        // Track no-progress iterations to prevent infinite churn
        if made_progress {
            no_progress_count = 0;
        } else {
            no_progress_count += 1;
            if no_progress_count >= settings.max_no_progress {
                println!(
                    "⚠️ No progress for {} iterations, stopping",
                    no_progress_count
                );
                logger.warning(&format!("No progress for {} iterations", no_progress_count));
                break;
            }
        }

        // Display token usage
        if let Some(ref stats) = stats::fetch_token_stats() {
            display::display_token_stats(stats);
        }

        // --- Step 5: Handle Loop Continuation ---
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

    // --- Final Result ---
    if last_run_success {
        Ok(())
    } else {
        anyhow::bail!("Autonomous run complete but the last feature failed verification.")
    }
}
