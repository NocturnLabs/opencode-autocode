//! Autonomous agent runner
//!
//! Replaces run-autonomous.sh with a native Rust implementation.
//! Runs OpenCode in batch mode with automatic session continuation.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::regression;

/// Run result from a single OpenCode session
#[derive(Debug)]
#[allow(dead_code)]
enum SessionResult {
    /// Session completed successfully, continue to next
    Continue,
    /// All tests passing, project complete
    Complete,
    /// Error occurred, stop
    Error(String),
    /// Stop signal detected
    Stopped,
}

/// Run the autonomous agent loop
pub fn run(limit: Option<usize>, config_path: Option<&Path>) -> Result<()> {
    // Load configuration
    let config = match config_path {
        Some(path) => Config::load_from_file(path)?,
        None => Config::load(None)?,
    };

    let delay = config.autonomous.delay_between_sessions;
    let max_iterations = if config.autonomous.max_iterations == 0 {
        limit.unwrap_or(usize::MAX)
    } else {
        limit.unwrap_or(config.autonomous.max_iterations as usize)
    };
    let model = &config.models.autonomous;
    let log_level = &config.autonomous.log_level;
    let feature_list_file = &config.paths.feature_list_file;

    println!("═══════════════════════════════════════════════════");
    println!("  OpenCode Autonomous Agent Runner");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("Project directory: {}", std::env::current_dir()?.display());
    println!(
        "Max iterations: {}",
        if max_iterations == usize::MAX {
            "unlimited".to_string()
        } else {
            max_iterations.to_string()
        }
    );
    println!("Model: {}", model);
    println!("Delay between sessions: {}s", delay);
    println!();
    println!("Sessions will run in batch mode and continue automatically.");
    println!("Press Ctrl+C to stop.");
    println!();

    let mut iteration = 0;
    let session_id: Option<String> = None;

    loop {
        iteration += 1;

        // Check max iterations
        if iteration > max_iterations {
            println!();
            println!("Reached max iterations ({})", max_iterations);
            break;
        }

        println!();
        println!("═══════════════════════════════════════════════════");
        println!(
            "  Session {} - {}",
            iteration,
            chrono::Local::now().format("%H:%M:%S")
        );
        println!("═══════════════════════════════════════════════════");
        println!();

        // Determine command based on feature_list.json existence
        let feature_path = Path::new(feature_list_file);
        let command = if !feature_path.exists() {
            println!("→ First run: auto-init");
            "auto-init"
        } else {
            // Count remaining tests
            let (passing, remaining) = count_feature_status(feature_path)?;
            println!("→ Progress: {} passing, {} remaining", passing, remaining);

            if remaining == 0 && passing > 0 {
                println!();
                println!("🎉 All tests passing! Project complete!");
                break;
            }

            "auto-continue"
        };

        println!("→ Running: opencode run --command /{}", command);
        println!();

        // Execute opencode
        let result = run_opencode_session(command, model, log_level, session_id.as_deref())?;

        match result {
            SessionResult::Continue => {
                println!("→ Session complete, continuing...");
                println!("→ Next session in {}s (Ctrl+C to stop)", delay);
                thread::sleep(Duration::from_secs(delay as u64));
            }
            SessionResult::Complete => {
                println!();
                println!("🎉 All tests passing! Project complete!");
                break;
            }
            SessionResult::Error(msg) => {
                println!();
                println!("⚠ OpenCode exited with error: {}", msg);
                println!(
                    "Check logs and run manually: opencode run --command /{}",
                    command
                );
                break;
            }
            SessionResult::Stopped => {
                println!();
                println!("Stop signal detected (.opencode-stop file exists)");
                let _ = std::fs::remove_file(".opencode-stop");
                break;
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  Runner stopped");
    println!("═══════════════════════════════════════════════════");
    println!();

    // Print final status
    let feature_path = Path::new(feature_list_file);
    if feature_path.exists() {
        let (passing, remaining) = count_feature_status(feature_path)?;
        println!(
            "Status: {} / {} tests passing",
            passing,
            passing + remaining
        );
    }

    println!();
    println!("To resume: opencode-autocode autonomous");
    println!("To stop:   touch .opencode-stop");

    Ok(())
}

/// Count passing and failing features from feature_list.json
fn count_feature_status(path: &Path) -> Result<(usize, usize)> {
    let features = regression::parse_feature_list(path)?;
    let passing = features.iter().filter(|f| f.passes).count();
    let remaining = features.iter().filter(|f| !f.passes).count();
    Ok((passing, remaining))
}

/// Run a single OpenCode session
fn run_opencode_session(
    command: &str,
    model: &str,
    log_level: &str,
    session_id: Option<&str>,
) -> Result<SessionResult> {
    // Check for stop signal before running
    if Path::new(".opencode-stop").exists() {
        return Ok(SessionResult::Stopped);
    }

    let mut cmd = Command::new("opencode");
    cmd.arg("run")
        .arg("--command")
        .arg(command)
        .arg("--model")
        .arg(model)
        .arg("--log-level")
        .arg(log_level);

    if let Some(sid) = session_id {
        cmd.arg("--session").arg(sid);
        println!("→ Continuing session: {}", sid);
    }

    let status = cmd.status().context("Failed to execute opencode command")?;

    println!();
    println!(
        "→ OpenCode exited with code: {}",
        status.code().unwrap_or(-1)
    );

    if !status.success() {
        return Ok(SessionResult::Error(format!(
            "exit code {}",
            status.code().unwrap_or(-1)
        )));
    }

    // Check for stop signal after running
    if Path::new(".opencode-stop").exists() {
        return Ok(SessionResult::Stopped);
    }

    Ok(SessionResult::Continue)
}
