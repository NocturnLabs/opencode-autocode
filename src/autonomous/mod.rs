//! Autonomous agent runner
//!
//! Runs OpenCode in batch mode with automatic session continuation
//! until all features pass.

pub mod debug_logger;
mod display;
mod features;
mod git;
pub mod parallel;
mod session;
mod settings;
mod webhook;

use anyhow::Result;
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

/// Run parallel workers using git worktrees
pub fn run_parallel(
    worker_count: usize,
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
) -> Result<()> {
    debug_logger::init(developer_mode);
    let logger = debug_logger::get();
    let config = load_config(config_path)?;
    let settings = settings::LoopSettings::from_config(&config, limit);
    let db_path = Path::new(&settings.database_file);

    let mut iteration = 0usize;

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            logger.info("Reached max iterations");
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        if session::stop_signal_exists() {
            logger.info("Parallel Coordinator: Stop signal received.");
            break;
        }

        // Get pending features
        let pending = features::get_pending_features(db_path, worker_count)?;
        if pending.is_empty() {
            println!("✅ No pending features to work on");
            break;
        }

        logger.separator();
        logger.info(&format!("Parallel Iteration {} starting", iteration));
        display::display_session_header(iteration);

        println!("📋 Selected {} features for parallel work:", pending.len());
        for f in &pending {
            println!("   • #{}: {}", f.id.unwrap_or(0), f.description);
        }

        let base_path = std::env::current_dir()?;
        let mut coordinator = parallel::Coordinator::new(worker_count, base_path.clone());

        // Create worktrees and spawn workers
        let mut handles = Vec::new();
        for feature in pending {
            let (worktree_path, branch_name) = parallel::create_worktree(&feature, &base_path)?;
            println!("🌳 Created worktree: {}", branch_name);

            let feature_id = feature.id.unwrap_or(0);
            let wt = worktree_path.clone();
            let bn = branch_name.clone();

            // Spawn worker thread
            let handle = std::thread::spawn(move || {
                let status = std::process::Command::new("opencode-autocode")
                    .args([
                        "vibe",
                        "--limit",
                        "1",
                        "--feature-id",
                        &feature_id.to_string(),
                    ])
                    .current_dir(&wt)
                    .status();

                parallel::WorkerResult {
                    feature_id,
                    branch_name: bn,
                    worktree_path: wt,
                    success: status.map(|s| s.success()).unwrap_or(false),
                }
            });
            handles.push(handle);
        }

        // Wait for workers and queue results
        for handle in handles {
            if let Ok(result) = handle.join() {
                println!(
                    "{}  Worker {} finished ({})",
                    if result.success { "✅" } else { "❌" },
                    result.feature_id,
                    if result.success { "success" } else { "failed" }
                );
                coordinator.queue_for_merge(result);
            }
        }

        // Process merge queue
        println!("\n📦 Processing merge queue...");
        let merged = coordinator.process_merge_queue()?;
        println!("✅ Merged {} features to main", merged);

        logger.info(&format!(
            "Parallel iteration complete: {} workers, {} merged",
            worker_count, merged
        ));

        // Delay between iterations
        if iteration < settings.max_iterations {
            std::thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
        }
    }

    Ok(())
}

/// Run the autonomous agent loop
pub fn run(
    limit: Option<usize>,
    config_path: Option<&Path>,
    developer_mode: bool,
    single_model: bool,
    enhancement_mode: bool,
    target_feature_id: Option<i64>,
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

    run_supervisor_loop(&config, &settings, enhancement_mode, target_feature_id)?;

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
                    let feature_opt = if let Some(id) = target_feature_id {
                        features::get_feature_by_id(db_path, id)?
                    } else {
                        features::get_first_pending_feature(db_path)?
                    };

                    if let Some(feature) = feature_opt {
                        generate_continue_template(&feature)?;
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

                // Generate dynamic auto-fix template
                generate_fix_template(&feature, &error, db_path)?;
                "auto-fix-active".to_string()
            }
        };

        println!("→ Running: opencode run --command /{}", command_name);
        println!();

        // 2. Get the feature the agent SHOULD be working on (for verification after session)
        let target_feature = if let Some(id) = target_feature_id {
            features::get_feature_by_id(db_path, id)?
        } else {
            features::get_first_pending_feature(db_path)?
        };

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
                    last_run_success = true;

                    // Mark as passing
                    let db = crate::db::Database::open(db_path)?;
                    db.features().mark_passing(&feature.description)?;
                    println!("  ✓ Marked as passing in DB");

                    // NEW: Mark in Conductor plan if active track matches
                    if let Some(track) = conductor::get_active_track(config)? {
                        let plan_path = track.path.join("plan.md");
                        if let Ok(tasks) = conductor::parse_plan(&plan_path) {
                            if let Some(task) = conductor::get_next_task(&tasks) {
                                // Only mark if the description matches or it's the next logical task
                                let _ = conductor::mark_task_complete(&plan_path, task.line_number);
                                println!(
                                    "  ✓ Marked task complete in plan.md: {}",
                                    task.description
                                );
                            }
                        }
                    }

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

                    // Mark as failing with error context for auto-fix
                    let db = crate::db::Database::open(db_path)?;
                    db.features()
                        .mark_failing_with_error(&feature.description, Some(&error_msg))?;
                    println!("  → Feature marked as failing (will auto-fix next iteration)");

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

fn determine_action(
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
Implement this feature completely:

**Feature #{}: {}**

## Acceptance Criteria
{}

## What You Do
1. Implement the feature with production-quality code
2. Write necessary tests if applicable
3. **VERIFY** that the verification command below is still correct for your implementation.
4. If the command changed (e.g. new test file path), you **MUST** update it in the database:
   `opencode-autocode db exec "UPDATE features SET verification_command = 'your-new-command' WHERE id = {}"`
5. Output `===SESSION_COMPLETE===` when implementation is done

## What Supervisor Does (NOT YOU)
The supervisor will automatically handle after your session:
- Run verification: `{}`
- Commit changes to git
- Mark feature as passing if verification succeeds

## Rules
- Do NOT run git commands (git add, git commit, git push)
- Do NOT run the verification command Yourself
- Do NOT call mark-pass
- **ALLOWED**: You may use `opencode-autocode db` commands to update your OWN feature's verification_command or steps.
- ONLY implement this one feature and output ===SESSION_COMPLETE===
"#,
        feature.id.unwrap_or(0),
        feature.description,
        if feature.steps.is_empty() {
            "Not specified - implement as described".to_string()
        } else {
            feature
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        },
        feature.id.unwrap_or(0),
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
