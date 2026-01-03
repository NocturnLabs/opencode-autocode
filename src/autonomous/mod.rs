//! Autonomous agent runner
//!
//! Runs OpenCode in batch mode with automatic session continuation
//! until all features pass.

pub mod debug_logger;
mod display;
mod features;
mod git;
mod session;
mod settings;
mod webhook;

use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::conductor;
use crate::config::Config;
use crate::regression;

use features::FeatureProgress;
use settings::{handle_session_result, LoopAction, LoopSettings};

/// Actions determined by the Supervisor
enum SupervisorAction {
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

/// Run the autonomous agent loop
pub fn run(
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
    single_model: bool,
    enhancement_mode: bool,
) -> Result<()> {
    // Initialize debug logger
    debug_logger::init(developer_mode);
    let logger = debug_logger::get();

    let config = load_config(config_path)?;
    let settings = LoopSettings::from_config(&config, limit);

    // Register Ctrl+C handler to create stop signal file
    ctrlc::set_handler(|| {
        std::fs::write(session::STOP_SIGNAL_FILE, "").ok();
        println!("\n→ Ctrl+C detected, stopping after current session...");
    })
    .ok();

    logger.separator();
    logger.info("OpenCode Supervisor starting");
    logger.info(&format!("Developer mode: {}", developer_mode));
    logger.info(&format!(
        "Project directory: {}",
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    ));
    logger.info(&format!("Model: {}", settings.model));
    logger.info(&format!(
        "Dual-model: {}",
        if single_model {
            "disabled (--single-model)"
        } else {
            "enabled (reasoning + @coder)"
        }
    ));
    logger.info(&format!(
        "Max iterations: {}",
        if settings.max_iterations == usize::MAX {
            "unlimited".to_string()
        } else {
            settings.max_iterations.to_string()
        }
    ));
    logger.info(&format!(
        "Session timeout: {} minutes",
        settings.session_timeout
    ));
    logger.separator();

    display::display_banner(
        &settings.model,
        settings.max_iterations,
        settings.delay_seconds,
        developer_mode,
    );

    run_supervisor_loop(&config, &settings, enhancement_mode)?;

    // Final status display
    let db_path = Path::new(&settings.database_file);
    let (passing, total) = if db_path.exists() {
        FeatureProgress::load_from_db(db_path)
            .map(|p| (p.passing, p.total()))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };
    logger.separator();
    logger.info(&format!(
        "Supervisor stopped. Final status: {}/{} tests passing",
        passing, total
    ));
    logger.separator();

    display::display_final_status(passing, total, developer_mode);

    Ok(())
}

fn load_config(config_path: Option<&Path>) -> Result<Config> {
    match config_path {
        Some(path) => Config::load_from_file(path),
        None => Config::load(None),
    }
}

fn run_supervisor_loop(
    config: &Config,
    settings: &LoopSettings,
    enhancement_mode: bool,
) -> Result<()> {
    let db_path = Path::new(&settings.database_file);
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;
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
        let action = determine_action(db_path, config)?;

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
                match handle_enhancement_phase(db_path, config, settings, iteration) {
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
                    if let Some(feature) = features::get_first_pending_feature(db_path)? {
                        generate_continue_template(&feature)?;
                        println!(
                            "📋 Feature #{}: {}",
                            feature.id.unwrap_or(0), feature.description
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

                // Generate dynamic auto-fix template
                generate_fix_template(&feature, &error, db_path)?;
                "auto-fix-active".to_string()
            }
        };

        println!("→ Running: opencode run --command /{}", command_name);
        println!();

        // 2. Get the feature the agent SHOULD be working on (for verification after session)
        let target_feature = features::get_first_pending_feature(db_path)?;

        // 3. Run Session
        let result = session::execute_opencode_session(
            &command_name,
            &settings.model,
            &settings.log_level,
            None,
            settings.session_timeout,
            logger,
        )?;

        // 4. Supervisor Verification (agent does NOT mark-pass, we do it)
        if let Some(feature) = target_feature {
            println!("🔍 Supervisor: Verifying feature...");
            println!("   Feature: {}", feature.description);

            if let Some(cmd) = &feature.verification_command {
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output()?;

                if output.status.success() {
                    println!("  ✅ Verification PASSED!");

                    // Mark as passing
                    let db = crate::db::Database::open(db_path)?;
                    db.features().mark_passing(&feature.description)?;
                    println!("  ✓ Marked as passing");

                    // Commit if needed
                    if settings.auto_commit {
                        let _ =
                            git::commit_completed_feature(&feature.description, settings.verbose);
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
                    if !stderr.is_empty() {
                        println!("     Error: {}", stderr.lines().next().unwrap_or(""));
                    }
                    // Feature remains in failing state, agent will retry next session
                }
            } else {
                println!("  ⚠ No verification command (manual check required)");
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

    Ok(())
}

/// Types of verification failures - determines corrective action
#[derive(Debug, PartialEq)]
enum VerificationFailure {
    /// Filter/grep didn't match any tests (command is wrong, not code)
    NoTestsMatch,
    /// Test file doesn't exist
    TestFileMissing,
    /// Command itself is invalid (missing binary, syntax error)
    CommandError,
    /// Actual test assertion failure (real regression)
    AssertionFailure,
}

impl VerificationFailure {
    fn as_str(&self) -> &'static str {
        match self {
            Self::NoTestsMatch => "no tests matched filter",
            Self::TestFileMissing => "test file missing",
            Self::CommandError => "command error",
            Self::AssertionFailure => "assertion failure",
        }
    }
}

/// Classify a verification failure based on error output
fn classify_verification_failure(error: &str) -> VerificationFailure {
    let lower = error.to_lowercase();

    // Patterns that indicate the verification command is broken, not the code
    if lower.contains("no test files")
        || lower.contains("did not match any")
        || lower.contains("filters did not match")
        || lower.contains("pattern not found")
        || lower.contains("no tests found")
        || lower.contains("no specs found")
    {
        return VerificationFailure::NoTestsMatch;
    }

    if lower.contains("cannot find")
        || lower.contains("no such file")
        || lower.contains("file not found")
        || lower.contains("enoent")
    {
        return VerificationFailure::TestFileMissing;
    }

    if lower.contains("command not found")
        || lower.contains("unknown command")
        || lower.contains("not recognized")
        || lower.contains("spawn unknown")
        || lower.contains("permission denied")
    {
        return VerificationFailure::CommandError;
    }

    // Default: assume actual test failure (code issue)
    VerificationFailure::AssertionFailure
}

fn determine_action(db_path: &Path, config: &Config) -> Result<SupervisorAction> {
    let logger = debug_logger::get();

    // 0. REGRESSION CHECK (Priority #1)
    if FeatureProgress::has_features(db_path) {
        let db = crate::db::Database::open(db_path)?;
        let features = db.features().list_all()?;

        // Run check on ALL features (not just passing ones to be safe, but regression checks usually imply passing ones)
        // actually regression check only checks passing ones.
        // We want to verify that previously passing features are STILL passing.
        let summary = regression::run_regression_check(&features, None, false)?;

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

fn generate_fix_template(
    feature: &crate::db::features::Feature,
    error: &str,
    _db_path: &Path,
) -> Result<()> {
    // Read template
    let template_path = Path::new("templates/commands/auto-fix.md");
    let template = if template_path.exists() {
        std::fs::read_to_string(template_path)?
    } else {
        // Fallback
        "# Regression Fix\nFix {{failing_feature}}\nError: {{error_message}}".to_string()
    };

    // Replace variables
    let content = template
        .replace("{{failing_feature}}", &feature.description)
        .replace("{{error_message}}", error)
        .replace("{{current_feature}}", "latest changes")
        .replace(
            "{{verification_command}}",
            feature.verification_command.as_deref().unwrap_or("unknown"),
        );

    // Write to active command file
    let target = Path::new(".opencode/command/auto-fix-active.md");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, content)?;
    Ok(())
}

/// Generate a minimal continue template with feature context injected by supervisor.
/// This removes LLM responsibility for querying the database.
fn generate_continue_template(feature: &crate::db::features::Feature) -> Result<()> {
    let content = format!(
        r#"# Implement Feature

## Your Task
Implement the following feature completely:

**Feature #{}: {}**

## Acceptance Criteria
{}

## Verification Command
After implementation, this command should pass (supervisor will run it):
```bash
{}
```

## Guidelines
1. Implement the feature with production-quality code
2. Write necessary tests
3. Ensure the verification command passes
4. Commit your changes with a descriptive message
5. Output `===SESSION_COMPLETE===` when done

Do NOT:
- Query the database for feature information (already provided above)
- Call `mark-pass` (supervisor handles this)
- Work on any other features (one feature per session)
"#,
        feature.id.unwrap_or(0),
        feature.description,
        if feature.steps.is_empty() {
            "Not specified - implement as described".to_string()
        } else {
            feature.steps.iter().enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        },
        feature
            .verification_command
            .as_deref()
            .unwrap_or("# No verification command specified")
    );

    let target = Path::new(".opencode/command/auto-continue-active.md");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, content)?;
    Ok(())
}

#[allow(dead_code)]
fn verify_and_commit(
    feature_desc: &str,
    db_path: &Path,
    config: &Config,
    settings: &LoopSettings,
    session_num: usize,
) -> Result<()> {
    let db = crate::db::Database::open(db_path)?;
    let features = db.features().list_all()?;
    let feature = features
        .iter()
        .find(|f| f.description == feature_desc)
        .context("Feature not found in DB")?;

    // 1. Run Verification
    println!("  • Verifying '{}'...", feature.description);
    if let Some(cmd) = &feature.verification_command {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()?;

        if !output.status.success() {
            println!("  ❌ Verification FAILED!");
            println!("     Command: {}", cmd);
            println!("     Code: {}", output.status.code().unwrap_or(-1));

            // ROLLBACK STATUS
            db.features().mark_failing(&feature.description)?;
            println!("  ↺ Rolled back status to 'failing'");
            return Ok(()); // Do not commit, do not notify success
        }
        println!("  ✅ Verified!");
    } else {
        println!("  ⚠ No verification command (manual verify)");
    }

    // 2. Commit
    if settings.auto_commit {
        let _ = git::commit_completed_feature(&feature.description, settings.verbose);
    }

    // 3. Notify
    let progress = FeatureProgress::load_from_db(db_path)?;
    let _ = webhook::notify_feature_complete(
        config,
        feature,
        session_num,
        progress.passing,
        progress.total(),
    );

    Ok(())
}

fn handle_enhancement_phase(
    _db_path: &Path,
    _config: &Config,
    _settings: &LoopSettings,
    _iteration: usize,
) -> Result<LoopAction> {
    println!("\n✨ All features complete! The autonomous loop is now in enhancement mode.");
    println!("What would you like to enhance? (or type 'exit' to finish)");
    print!("> ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut enhancement_request = String::new();
    reader.read_line(&mut enhancement_request)?;

    let enhancement_request = enhancement_request.trim();

    if enhancement_request.is_empty() || enhancement_request.to_lowercase() == "exit" {
        return Ok(LoopAction::Break);
    }

    // Generate dynamic enhancement template
    let template = r#"# Enhancement Request
{{enhancement_request}}

Please implement this enhancement for the current project.
"#;
    let content = template.replace("{{enhancement_request}}", enhancement_request);

    // Write to active command file
    let target = Path::new(".opencode/command/auto-enhance-active.md");
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(target, content)?;

    Ok(LoopAction::Continue)
}
