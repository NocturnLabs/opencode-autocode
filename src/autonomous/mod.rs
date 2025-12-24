//! Autonomous agent runner
//!
//! Runs OpenCode in batch mode with automatic session continuation
//! until all features pass.

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

use crate::config::Config;
use crate::regression;

use features::FeatureProgress;
use settings::{handle_session_result, LoopAction, LoopSettings};

/// Run the autonomous agent loop
pub fn run(limit: Option<usize>, config_path: Option<&Path>, developer_mode: bool) -> Result<()> {
    let config = load_config(config_path)?;
    let settings = LoopSettings::from_config(&config, limit);

    display::display_banner(
        &settings.model,
        settings.max_iterations,
        settings.delay_seconds,
        developer_mode,
    );

    run_main_loop(&config, &settings)?;

    let feature_path = Path::new(&settings.feature_list_file);
    let (passing, total) = if feature_path.exists() {
        FeatureProgress::load(feature_path)
            .map(|p| (p.passing, p.total()))
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };
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
    let feature_path = Path::new(&settings.feature_list_file);
    let mut iteration = 0usize;
    let mut consecutive_errors = 0u32;

    loop {
        iteration += 1;

        if iteration > settings.max_iterations {
            println!("\nReached max iterations ({})", settings.max_iterations);
            break;
        }

        display::display_session_header(iteration);

        let command = determine_command(feature_path)?;
        if command.is_none() {
            println!("\n🎉 All tests passing! Project complete!");
            break;
        }
        let command = command.unwrap();

        println!("→ Running: opencode run --command /{}", command);
        println!();

        let before_passing = features::get_passing_feature_descriptions(feature_path)?;

        let result = session::execute_opencode_session(
            command,
            &settings.model,
            &settings.log_level,
            None,
            settings.session_timeout,
        )?;

        let after_passing = features::get_passing_feature_descriptions(feature_path)?;
        let new_features = features::detect_newly_completed(&before_passing, &after_passing);

        handle_completed_features(config, settings, &new_features, feature_path, iteration)?;

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

fn determine_command(feature_path: &Path) -> Result<Option<&'static str>> {
    if !feature_path.exists() {
        println!("→ First run: auto-init");
        return Ok(Some("auto-init"));
    }

    let progress = FeatureProgress::load(feature_path)?;
    println!(
        "→ Progress: {} passing, {} remaining",
        progress.passing, progress.remaining
    );

    if progress.all_passing() {
        return Ok(None);
    }

    Ok(Some("auto-continue"))
}

fn handle_completed_features(
    config: &Config,
    settings: &LoopSettings,
    new_features: &[String],
    feature_path: &Path,
    session_number: usize,
) -> Result<()> {
    if new_features.is_empty() {
        return Ok(());
    }

    let progress =
        FeatureProgress::load(feature_path).unwrap_or(FeatureProgress { passing: 0, remaining: 0 });
    let features_list = regression::parse_feature_list(feature_path).ok();

    for feature_desc in new_features {
        if let Some(ref features) = features_list {
            if let Some(feature) = features.iter().find(|f| f.description == *feature_desc) {
                let _ = webhook::notify_feature_complete(
                    config,
                    feature,
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
