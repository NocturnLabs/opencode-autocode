//! Settings and result handling for the autonomous loop

use super::session;
use crate::config::{Config, McpConfig};
use anyhow::Result;
use std::path::Path;

/// Settings extracted from config for the main loop
pub struct LoopSettings {
    pub delay_seconds: u32,
    pub max_iterations: usize,
    pub enforce_max_iterations: bool,
    pub max_retries: u32,
    /// Warn after this many iterations without progress (0 = unlimited)
    pub max_no_progress: u32,
    pub model: String,

    pub log_level: String,
    pub database_file: String,
    pub session_timeout: u32,
    pub idle_timeout: u32,
    pub auto_commit: bool,
    pub verbose: bool,
    pub dual_model_enabled: bool,
    pub mcp: McpConfig,
}

impl LoopSettings {
    pub fn from_config(config: &Config, limit: Option<usize>) -> Self {
        let max_iterations = if config.autonomous.max_iterations == 0 {
            limit.unwrap_or(usize::MAX)
        } else {
            limit.unwrap_or(config.autonomous.max_iterations as usize)
        };
        let enforce_max_iterations = limit.is_some() || config.autonomous.max_iterations > 0;

        let db_path = Path::new(&config.paths.database_file);
        let database_file = if db_path.is_relative() {
            std::env::current_dir()
                .unwrap_or_default()
                .join(db_path)
                .to_string_lossy()
                .to_string()
        } else {
            config.paths.database_file.clone()
        };

        let max_no_progress = if config.autonomous.max_no_progress == 0 {
            u32::MAX
        } else {
            config.autonomous.max_no_progress
        };

        Self {
            delay_seconds: config.autonomous.delay_between_sessions,
            max_iterations,
            enforce_max_iterations,
            max_retries: config.agent.max_retry_attempts,
            max_no_progress,
            model: config.models.autonomous.clone(),
            log_level: config.autonomous.log_level.clone(),
            database_file,
            session_timeout: config.autonomous.session_timeout_minutes,
            idle_timeout: config.autonomous.idle_timeout_seconds,
            auto_commit: config.autonomous.auto_commit,
            verbose: config.ui.verbose,
            dual_model_enabled: true,
            mcp: config.mcp.clone(),
        }
    }
}

/// Action to take after handling a session result
pub enum LoopAction {
    Continue,
    Break,
    RetryWithBackoff(u32),
}

use super::session::SessionResult;

/// Handle the result of a session execution
pub fn handle_session_result(
    result: SessionResult,
    settings: &LoopSettings,
    consecutive_errors: &mut u32,
) -> LoopAction {
    match result {
        SessionResult::Continue => {
            *consecutive_errors = 0;
            println!("→ Session complete, continuing...");
            println!(
                "→ Next session in {}s (Ctrl+C to stop)",
                settings.delay_seconds
            );
            LoopAction::Continue
        }

        SessionResult::EarlyTerminated { trigger } => {
            *consecutive_errors = 0;
            println!("⚠️ Session terminated early via pattern match");
            println!("   Trigger: {}", trigger);
            println!(
                "→ Next session in {}s (Ctrl+C to stop)",
                settings.delay_seconds
            );
            LoopAction::Continue
        }

        SessionResult::Error(msg) => {
            *consecutive_errors += 1;
            println!(
                "\n⚠ Session error (attempt {}/{}): {}",
                consecutive_errors, settings.max_retries, msg
            );

            if *consecutive_errors == settings.max_retries {
                println!(
                    "⚠️ Exceeded max retries ({}), continuing with backoff.",
                    settings.max_retries
                );
            }

            let exponent = (*consecutive_errors - 1).min(6);
            let backoff = settings.delay_seconds.saturating_mul(1 << exponent);
            println!("→ Retrying in {}s (exponential backoff)...", backoff);
            LoopAction::RetryWithBackoff(backoff)
        }
        SessionResult::Stopped => {
            println!("\nStop signal detected (.opencode-stop file exists)");
            super::session::clear_stop_signal();
            LoopAction::Break
        }
    }
}

fn load_config(config_path: Option<&Path>) -> Result<Config> {
    match config_path {
        Some(path) => Config::load_from_file(path),
        None => Config::load(None),
    }
}

/// Initialize a session, loading config and setting up logging
pub fn init_session(
    developer_mode: bool,
    config_path: Option<&Path>,
    limit: Option<usize>,
    single_model: bool,
    log_path: Option<&str>,
) -> Result<(Config, LoopSettings)> {
    crate::common::logging::init(developer_mode, log_path);
    let config = load_config(config_path)?;
    let mut settings = LoopSettings::from_config(&config, limit);
    settings.dual_model_enabled = !single_model;

    // Clear any lingering stop signal from a previous run
    session::clear_stop_signal();

    Ok((config, settings))
}
