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

        logger.separator();
        logger.info(&format!("Session {} starting", iteration));
        display::display_session_header(iteration);

        // 1. Determine Action (Supervisor Logic)
        let action = determine_action(db_path, config)?;

        let command_name = match action {
            SupervisorAction::Stop => {
                if enhancement_mode {
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
                } else {
                    logger.info("Supervisor: Stop signal received.");
                    break;
                }
            }
            SupervisorAction::Command(cmd) => {
                logger.info(&format!("Supervisor: Selected command '{}'", cmd));
                cmd.to_string()
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

        // 2. Run Session
        let before_passing = features::get_passing_feature_descriptions(db_path)?;

        // Execute the session
        let result = session::execute_opencode_session(
            &command_name,
            &settings.model,
            &settings.log_level,
            None,
            settings.session_timeout,
            logger,
        )?;

        // 3. Independent Verification (Trust but Verify)
        let after_passing = features::get_passing_feature_descriptions(db_path)?;
        // Detect what the agent CLAIMED to complete
        let claimed_new = features::detect_newly_completed(&before_passing, &after_passing);

        if !claimed_new.is_empty() {
            println!(
                "🔍 Supervisor: Verifying {} feature(s)...",
                claimed_new.len()
            );
            for feature_desc in &claimed_new {
                verify_and_commit(feature_desc, db_path, config, settings, iteration)?;
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

fn determine_action(db_path: &Path, config: &Config) -> Result<SupervisorAction> {
    let _logger = debug_logger::get();

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
                        return Ok(SupervisorAction::Fix {
                            feature: feature.clone(),
                            error: result.error_message.unwrap_or_default(),
                        });
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
