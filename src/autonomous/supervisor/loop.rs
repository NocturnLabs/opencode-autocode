use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::config::Config;

use crate::autonomous::decision::{determine_action, SupervisorAction};
use crate::autonomous::display;
use crate::autonomous::session;
use crate::autonomous::settings::{handle_session_result, LoopAction, LoopSettings};
use crate::autonomous::stats;

use crate::common::logging as debug_logger;

use super::actions::{prepare_command, ActionCommand};
use super::verification_step::perform_verification;

/// Runs the main supervisor loop.
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
            continue;
        }

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
                made_progress = perform_verification(
                    feature,
                    db_path,
                    config,
                    settings,
                    iteration,
                    &mut last_run_success,
                )?;
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
                // Smart sleep: skip delay if we made progress for faster iteration
                if made_progress {
                    // Clear terminal between sessions for clean debugging
                    print!("\x1b[2J\x1b[1;1H");
                    println!("🚀 Session {} complete, fast-forwarding...\n", iteration);
                } else {
                    println!(
                        "→ No progress, waiting {}s before next session...",
                        settings.delay_seconds
                    );
                    thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
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
        anyhow::bail!("Autonomous run complete but the last feature failed verification.")
    }
}
