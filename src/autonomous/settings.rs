//! Settings and result handling for the autonomous loop

use crate::config::Config;

/// Settings extracted from config for the main loop
pub struct LoopSettings {
    pub delay_seconds: u32,
    pub max_iterations: usize,
    pub max_retries: u32,
    pub model: String,
    pub log_level: String,
    pub feature_list_file: String,
    pub session_timeout: u32,
    pub auto_commit: bool,
    pub verbose: bool,
}

impl LoopSettings {
    pub fn from_config(config: &Config, limit: Option<usize>) -> Self {
        let max_iterations = if config.autonomous.max_iterations == 0 {
            limit.unwrap_or(usize::MAX)
        } else {
            limit.unwrap_or(config.autonomous.max_iterations as usize)
        };

        Self {
            delay_seconds: config.autonomous.delay_between_sessions,
            max_iterations,
            max_retries: config.agent.max_retry_attempts,
            model: config.models.autonomous.clone(),
            log_level: config.autonomous.log_level.clone(),
            feature_list_file: config.paths.feature_list_file.clone(),
            session_timeout: config.autonomous.session_timeout_minutes,
            auto_commit: config.autonomous.auto_commit,
            verbose: config.ui.verbose,
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
            println!("→ Next session in {}s (Ctrl+C to stop)", settings.delay_seconds);
            LoopAction::Continue
        }
        SessionResult::Complete => {
            println!("\n🎉 All tests passing! Project complete!");
            LoopAction::Break
        }
        SessionResult::Error(msg) => {
            *consecutive_errors += 1;
            println!(
                "\n⚠ Session error (attempt {}/{}): {}",
                consecutive_errors, settings.max_retries, msg
            );

            if *consecutive_errors >= settings.max_retries {
                println!("❌ Exceeded max retries ({}), stopping.", settings.max_retries);
                return LoopAction::Break;
            }

            let backoff = settings.delay_seconds * (1 << (*consecutive_errors - 1).min(4));
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
