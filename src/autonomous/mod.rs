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

use anyhow::Result;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::conductor;
use crate::config::Config;
use crate::regression;

use features::FeatureProgress;
use settings::{handle_session_result, LoopAction, LoopSettings};

/// Run the autonomous agent loop
pub fn run(limit: Option<usize>, config_path: Option<&Path>, developer_mode: bool) -> Result<()> {
    // Initialize debug logger
    debug_logger::init(developer_mode);
    let logger = debug_logger::get();

    let config = load_config(config_path)?;
    let settings = LoopSettings::from_config(&config, limit);

    logger.separator();
    logger.info("OpenCode Autonomous Agent Runner starting");
    logger.info(&format!("Developer mode: {}", developer_mode));
    logger.info(&format!(
        "Project directory: {}",
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    ));
    logger.info(&format!("Model: {}", settings.model));
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

    run_main_loop(&config, &settings)?;

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
        "Runner stopped. Final status: {}/{} tests passing",
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

fn run_main_loop(config: &Config, settings: &LoopSettings) -> Result<()> {
    let db_path = Path::new(&settings.database_file);
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;
    let logger = debug_logger::get();

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            logger.info(&format!(
                "Reached max iterations ({})",
                settings.max_iterations
            ));
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        logger.separator();
        logger.info(&format!(
            "Session {} starting at {}",
            iteration,
            chrono::Local::now().format("%H:%M:%S")
        ));
        display::display_session_header(iteration);

        let command = determine_command(db_path, config)?;
        if command.is_none() {
            logger.info("All tests passing! Project complete!");
            println!("\n🎉 All tests passing! Project complete!");
            break;
        }
        let command = command.unwrap();

        logger.info(&format!("Running command: /{}", command));
        println!("→ Running: opencode run --command /{}", command);
        println!();

        let before_passing = features::get_passing_feature_descriptions(db_path)?;

        let result = session::execute_opencode_session(
            command,
            &settings.model,
            &settings.log_level,
            None,
            settings.session_timeout,
            logger,
        )?;

        let after_passing = features::get_passing_feature_descriptions(db_path)?;
        let new_features = features::detect_newly_completed(&before_passing, &after_passing);

        if !new_features.is_empty() {
            for feature in &new_features {
                logger.info(&format!("Feature completed: \"{}\"", feature));
            }
        }

        handle_completed_features(config, settings, &new_features, db_path, iteration)?;

        // Display token usage after each session
        if let Some(ref stats) = session::fetch_token_stats() {
            logger.info(&format!(
                "Token usage: input={}, output={}, cost=${:.4}",
                stats.input_tokens, stats.output_tokens, stats.total_cost
            ));
            display::display_token_stats(stats);
        }

        match handle_session_result(result, settings, &mut consecutive_errors) {
            LoopAction::Continue => {
                logger.debug(&format!(
                    "Sleeping {}s before next session",
                    settings.delay_seconds
                ));
                thread::sleep(Duration::from_secs(settings.delay_seconds as u64));
            }
            LoopAction::Break => {
                logger.info("Loop terminated");
                break;
            }
            LoopAction::RetryWithBackoff(backoff) => {
                logger.info(&format!("Retrying with {}s backoff", backoff));
                thread::sleep(Duration::from_secs(backoff as u64));
            }
        }
    }

    Ok(())
}

fn determine_command(db_path: &Path, config: &Config) -> Result<Option<&'static str>> {
    let logger = debug_logger::get();

    // Phase 1: First run - auto-init populates features in the database
    if !FeatureProgress::has_features(db_path) {
        logger.info("Phase 1: First run, running auto-init");
        println!("→ First run: auto-init");
        return Ok(Some("auto-init"));
    }

    // Phase 2: Ensure conductor context exists (if auto_setup enabled)
    if config.conductor.auto_setup && !conductor::context_exists(config) {
        logger.info("Phase 2: Conductor context missing, running auto-context");
        println!("→ Setting up project context: auto-context");
        return Ok(Some("auto-context"));
    }

    // Phase 3: Check for active track with incomplete tasks
    if let Some(track) = conductor::get_active_track(config)? {
        let plan_path = track.path.join("plan.md");
        if let Ok(tasks) = conductor::parse_plan(&plan_path) {
            if let Some(next_task) = conductor::get_next_task(&tasks) {
                logger.info(&format!(
                    "Phase 3: Continuing track '{}', next task: {}",
                    track.name, next_task.description
                ));
                println!(
                    "→ Active track: {} (next: {})",
                    track.name, next_task.description
                );
                return Ok(Some("auto-continue"));
            }
        }
    }

    // Phase 4: Check database feature progress
    let progress = FeatureProgress::load_from_db(db_path)?;
    println!(
        "→ Progress: {} passing, {} remaining",
        progress.passing, progress.remaining
    );

    if progress.all_passing() {
        logger.info("Phase 4: All features passing, project complete");
        return Ok(None);
    }

    // Phase 5: No active track, but features remain - use auto-continue
    // (The AI in auto-continue will pick the next failing feature)
    logger.info("Phase 5: No active track, running auto-continue for next feature");
    Ok(Some("auto-continue"))
}

fn handle_completed_features(
    config: &Config,
    settings: &LoopSettings,
    new_features: &[String],
    db_path: &Path,
    session_number: usize,
) -> Result<()> {
    if new_features.is_empty() {
        return Ok(());
    }

    let progress = FeatureProgress::load_from_db(db_path).unwrap_or(FeatureProgress {
        passing: 0,
        remaining: 0,
    });

    // Load features from database for webhook notifications
    let features_list = if db_path.exists() {
        crate::db::Database::open(db_path)
            .ok()
            .and_then(|db| db.features().list_all().ok())
    } else {
        None
    };

    for feature_desc in new_features {
        if let Some(ref features) = features_list {
            if let Some(feature) = features.iter().find(|f| f.description == *feature_desc) {
                // Convert db::Feature to regression::Feature for webhook
                let regression_feature = regression::Feature {
                    category: feature.category.clone(),
                    description: feature.description.clone(),
                    steps: feature.steps.clone(),
                    passes: feature.passes,
                    verification_command: feature.verification_command.clone(),
                };

                let _ = webhook::notify_feature_complete(
                    config,
                    &regression_feature,
                    session_number,
                    progress.passing,
                    progress.total(),
                );

                if settings.auto_commit {
                    let _ = git::commit_completed_feature(&feature.description, settings.verbose);
                }
            }
        }
    }

    Ok(())
}
