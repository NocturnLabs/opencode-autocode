use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::db::features::Feature;

use crate::autonomous::alternative;
use crate::autonomous::decision::{determine_action, SupervisorAction};
use crate::autonomous::display;
use crate::autonomous::session;
use crate::autonomous::settings::{handle_session_result, LoopAction, LoopSettings};
use crate::autonomous::stats;
use crate::autonomous::webhook::{notify_failure, FailureReason};

use crate::common::logging as debug_logger;

use super::actions::{prepare_command, ActionCommand};
use super::two_phase::{execute_coding_phase, execute_reasoning_phase, ReasoningResult};
use super::verification_step::perform_verification;

/// Execute a feature using two-phase orchestration (reasoning → coding)
fn execute_two_phase_feature(
    feature: &Feature,
    config: &Config,
    settings: &LoopSettings,
    iteration: &mut usize,
    logger: &debug_logger::DebugLogger,
) -> Result<session::SessionResult> {
    logger.separator();
    logger.info(&format!(
        "Two-phase orchestration for feature #{}: {}",
        feature.id.unwrap_or(0),
        feature.description
    ));

    // Phase 1: Reasoning
    let reasoning_result = execute_reasoning_phase(feature, config, settings, logger)?;

    match reasoning_result {
        ReasoningResult::Success(packet) => {
            println!("\n✓ Reasoning phase produced valid implementation packet");
            logger.info(&format!(
                "Implementation packet has {} files, {} edits, {} commands",
                packet.files_to_modify.len(),
                packet.edits.len(),
                packet.commands_to_run.len()
            ));

            // Phase 2: Coding
            let coding_result = execute_coding_phase(&packet, feature, settings, logger)?;
            Ok(coding_result)
        }
        ReasoningResult::InvalidJson(msg) => {
            println!("\n❌ Reasoning phase failed: Invalid JSON");
            println!("   Error: {}", msg);
            logger.error(&format!("Invalid JSON from reasoning: {}", msg));

            // Retry reasoning phase with simpler prompt (up to max_retry_attempts)
            if *iteration < settings.max_retries as usize {
                println!("→ Retrying reasoning phase...");
                thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
                *iteration += 1;
                execute_two_phase_feature(feature, config, settings, iteration, logger)
            } else {
                println!("⚠️ Max retries exceeded, falling back to single-phase");
                // Fall back to traditional single-phase session
                crate::autonomous::templates::generate_continue_template(
                    feature, config, false, // No @coder references
                )?;
                logger.info("Falling back to single-phase implementation");

                session::execute_opencode_session(
                    session::SessionOptions {
                        command: "auto-continue-active".to_string(),
                        model: settings.coding_model.clone(),
                        log_level: settings.log_level.clone(),
                        session_id: None,
                        timeout_minutes: settings.session_timeout,
                        idle_timeout_seconds: settings.idle_timeout,
                        opencode_path: settings.opencode_path.clone(),
                    },
                    logger,
                )
            }
        }
        ReasoningResult::ValidationError(msg) => {
            println!("\n❌ Reasoning phase failed: Validation error");
            println!("   Error: {}", msg);
            logger.error(&format!("Validation error from reasoning: {}", msg));
            Ok(session::SessionResult::Error(msg))
        }
        ReasoningResult::Error(msg) => {
            println!("\n❌ Reasoning phase failed: {}", msg);
            logger.error(&format!("Reasoning phase error: {}", msg));
            Ok(session::SessionResult::Error(msg))
        }
    }
}

/// Runs the main supervisor loop.
pub fn run_supervisor_loop(
    config: &Config,
    settings: &LoopSettings,
    enhancement_mode: bool,
    target_feature_id: Option<i64>,
    banner_width: usize,
) -> Result<()> {
    let db_path = Path::new(&settings.database_file);
    let logger = debug_logger::get();

    // Mutable state for the loop
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;
    let mut no_progress_count = 0u32;
    let mut last_run_success = true;
    let mut alternative_attempts: HashMap<String, u32> = HashMap::new();
    let mut last_error_context: Option<String> = None;

    // --- Main Loop (Bounded by max_iterations) ---
    loop {
        iteration += 1;

        // --- Exit Condition 1: Max Iterations ---
        if iteration > settings.max_iterations {
            if settings.enforce_max_iterations {
                logger.info("Reached max iterations; stopping as requested");
                println!("\nReached max iterations ({})", settings.max_iterations);
                let _ = notify_failure(
                    config,
                    FailureReason::MaxIterations {
                        iterations: settings.max_iterations,
                    },
                );
                break;
            }

            logger.info("Reached max iterations; continuing until user stop");
            println!(
                "\nReached max iterations ({}), continuing until user stop",
                settings.max_iterations
            );
            let _ = notify_failure(
                config,
                FailureReason::MaxIterations {
                    iterations: settings.max_iterations,
                },
            );
        }

        // --- Exit Condition 2: Stop Signal ---
        if session::stop_signal_exists() {
            logger.info("Supervisor: Stop signal received.");
            break;
        }

        // --- Step 1: Determine Action ---
        let action = determine_action(db_path, config, target_feature_id)?;

        // Exit early if all features are complete (don't print a ghost session)
        if matches!(action, SupervisorAction::Complete) && !enhancement_mode {
            logger.info("Supervisor: All features complete.");
            break;
        }

        // Now safe to print the session header
        logger.separator();
        logger.info(&format!("Session {} starting", iteration));
        if config.ui.show_progress {
            display::display_session_header(iteration, banner_width);
        }

        // --- Step 2: Prepare Command ---
        let ActionCommand {
            name: command_name,
            active_feature,
            should_break,
            no_progress: action_no_progress,
        } = prepare_command(
            action,
            enhancement_mode,
            config,
            settings,
            &mut iteration,
            logger,
            target_feature_id,
        )?;

        if should_break {
            break;
        }

        if action_no_progress {
            no_progress_count += 1;
            maybe_generate_alternatives(
                config,
                active_feature.as_ref(),
                no_progress_count,
                last_error_context.as_deref(),
                &mut alternative_attempts,
            )?;
            if settings.max_no_progress != u32::MAX && no_progress_count == settings.max_no_progress
            {
                println!(
                    "⚠️ No progress for {} iterations, continuing with backoff",
                    no_progress_count
                );
                logger.warning(&format!("No progress for {} iterations", no_progress_count));
                let _ = notify_failure(
                    config,
                    FailureReason::NoProgress {
                        count: no_progress_count,
                        limit: settings.max_no_progress,
                    },
                );
            }
            let exponent = (no_progress_count.saturating_sub(1)).min(6);
            let factor = 1_u32.checked_shl(exponent).unwrap_or(u32::MAX);
            let backoff = settings.delay_seconds.saturating_mul(factor);
            println!("→ No actionable work, waiting {}s before retry...", backoff);
            thread::sleep(Duration::from_secs(backoff as u64));
            continue;
        }

        println!("→ Running: opencode run --command /{}", command_name);
        println!();

        // --- Step 3: Execute Session (with two-phase orchestration if applicable) ---
        let result = if !settings.single_model && active_feature.is_some() {
            // Two-phase orchestration for feature implementation
            if let Some(ref feature) = active_feature {
                execute_two_phase_feature(feature, config, settings, &mut iteration, logger)?
            } else {
                // Fallback to single session if no feature context
                session::execute_opencode_session(
                    session::SessionOptions {
                        command: command_name.to_string(),
                        model: if enhancement_mode {
                            settings.enhancement_model.clone()
                        } else {
                            settings.coding_model.clone()
                        },
                        log_level: settings.log_level.clone(),
                        session_id: None,
                        timeout_minutes: settings.session_timeout,
                        idle_timeout_seconds: settings.idle_timeout,
                        opencode_path: settings.opencode_path.clone(),
                    },
                    logger,
                )?
            }
        } else {
            // Single-model mode or enhancement mode - use traditional single session
            session::execute_opencode_session(
                session::SessionOptions {
                    command: command_name.to_string(),
                    model: if enhancement_mode {
                        settings.enhancement_model.clone()
                    } else {
                        settings.coding_model.clone()
                    },
                    log_level: settings.log_level.clone(),
                    session_id: None,
                    timeout_minutes: settings.session_timeout,
                    idle_timeout_seconds: settings.idle_timeout,
                    opencode_path: settings.opencode_path.clone(),
                },
                logger,
            )?
        };

        if let session::SessionResult::Error(msg) = &result {
            last_error_context = Some(msg.clone());
        }

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
                let outcome = perform_verification(
                    feature,
                    db_path,
                    config,
                    settings,
                    iteration,
                    &mut last_run_success,
                )?;
                made_progress = outcome.made_progress;
                if outcome.error_context.is_some() {
                    last_error_context = outcome.error_context;
                }
            } else {
                made_progress = true;
            }
        }

        // Track no-progress iterations to prevent infinite churn
        if made_progress {
            no_progress_count = 0;
        } else {
            no_progress_count += 1;
            maybe_generate_alternatives(
                config,
                active_feature.as_ref(),
                no_progress_count,
                last_error_context.as_deref(),
                &mut alternative_attempts,
            )?;
            if settings.max_no_progress != u32::MAX && no_progress_count == settings.max_no_progress
            {
                println!(
                    "⚠️ No progress for {} iterations, continuing with backoff",
                    no_progress_count
                );
                logger.warning(&format!("No progress for {} iterations", no_progress_count));
                let _ = notify_failure(
                    config,
                    FailureReason::NoProgress {
                        count: no_progress_count,
                        limit: settings.max_no_progress,
                    },
                );
            }
        }

        // Display token usage
        if config.ui.show_progress {
            if let Some(ref stats) = stats::fetch_token_stats() {
                display::display_token_stats(stats, banner_width);
            }
        }

        // --- Step 5: Handle Loop Continuation ---
        match handle_session_result(result, settings, &mut consecutive_errors) {
            LoopAction::Continue => {
                // Smart sleep: skip delay if we made progress for faster iteration
                if made_progress {
                    // Clear terminal between sessions for clean debugging
                    print!("\x1b[2J\x1b[1;1H");
                    println!("🚀 Session {} complete, fast-forwarding...\n", iteration);
                } else {
                    let exponent = (no_progress_count.saturating_sub(1)).min(6);
                    let factor = 1_u32.checked_shl(exponent).unwrap_or(u32::MAX);
                    let backoff = settings.delay_seconds.saturating_mul(factor);
                    println!("→ No progress, waiting {}s before next session...", backoff);
                    thread::sleep(Duration::from_secs(backoff as u64));
                }
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
        let _ = notify_failure(
            config,
            FailureReason::FatalError {
                message: "Last feature failed verification".to_string(),
            },
        );
        anyhow::bail!("Autonomous run complete but the last feature failed verification.")
    }
}

/// @param config Loaded configuration.
/// @param feature Active feature reference.
/// @param no_progress_count Current no-progress counter.
/// @param error_context Optional error context string.
/// @param attempt_tracker Tracks how many generations ran per feature.
/// @returns Result of generation attempt.
fn maybe_generate_alternatives(
    config: &Config,
    feature: Option<&Feature>,
    no_progress_count: u32,
    error_context: Option<&str>,
    attempt_tracker: &mut HashMap<String, u32>,
) -> Result<()> {
    if !config.alternative_approaches.enabled {
        return Ok(());
    }
    if config.alternative_approaches.retry_threshold == 0 {
        return Ok(());
    }
    if no_progress_count < config.alternative_approaches.retry_threshold {
        return Ok(());
    }

    let feature = match feature {
        Some(feature) => feature,
        None => return Ok(()),
    };

    let attempts = attempt_tracker
        .entry(feature.description.clone())
        .or_insert(0);
    let max_attempts = if config.agent.max_research_attempts == 0 {
        u32::MAX
    } else {
        config.agent.max_research_attempts
    };
    if *attempts >= max_attempts {
        return Ok(());
    }
    *attempts += 1;

    let cache_path = alternative::generate_alternative_approaches(config, feature, error_context)?;
    println!(
        "⚠️ Alternative approaches saved at {}",
        cache_path.display()
    );

    Ok(())
}
