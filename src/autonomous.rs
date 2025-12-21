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
    let mut consecutive_errors = 0u32;
    let max_retries = config.agent.max_retry_attempts;
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

        // Capture passing features before session
        let before_passing = get_passing_features(feature_path)?;

        // Execute opencode
        let result = run_opencode_session(command, model, log_level, session_id.as_deref())?;

        // Capture passing features after session
        let after_passing = get_passing_features(feature_path)?;

        // Find newly completed features
        let new_features = after_passing.difference(&before_passing);
        for feature_desc in new_features {
            // Find the full feature object
            if let Ok(features) = regression::parse_feature_list(feature_path) {
                if let Some(feature) = features
                    .into_iter()
                    .find(|f| f.description == *feature_desc)
                {
                    if let Err(e) = send_webhook_notification(&config, &feature) {
                        println!("⚠ Webhook error: {}", e);
                    }
                }
            }
        }

        match result {
            SessionResult::Continue => {
                consecutive_errors = 0; // Reset on success
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
                consecutive_errors += 1;
                println!();
                println!(
                    "⚠ Session error (attempt {}/{}): {}",
                    consecutive_errors, max_retries, msg
                );

                if consecutive_errors >= max_retries {
                    println!("❌ Exceeded max retries ({}), stopping.", max_retries);
                    break;
                }

                // Exponential backoff: delay * 2^(attempts-1)
                let backoff = delay * (1 << (consecutive_errors - 1).min(4));
                println!("→ Retrying in {}s (exponential backoff)...", backoff);
                thread::sleep(Duration::from_secs(backoff as u64));
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

/// Send a webhook notification for a completed feature
fn send_webhook_notification(config: &Config, feature: &regression::Feature) -> Result<()> {
    if !config.notifications.webhook_enabled {
        return Ok(());
    }

    let url = match &config.notifications.webhook_url {
        Some(u) => u,
        None => return Ok(()),
    };

    println!(
        "→ Sending webhook notification for: {}",
        feature.description
    );

    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown Project")
        .to_string();

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Prepare template data
    let mut data = std::collections::HashMap::new();
    data.insert("feature_name", feature.description.clone());
    data.insert("feature_description", feature.description.clone()); // Using description as name for now
    data.insert("project_name", project_name);
    data.insert("timestamp", timestamp);

    // Render template
    let handlebars = handlebars::Handlebars::new();
    let template_path = Path::new("templates/notifications/webhook.json");
    let template_content = if template_path.exists() {
        std::fs::read_to_string(template_path)?
    } else {
        // Fallback minimal template if file is missing
        r#"{ "content": "✅ Feature Completed: {{feature_name}} in {{project_name}}" }"#.to_string()
    };

    let rendered = handlebars
        .render_template(&template_content, &data)
        .context("Failed to render webhook template")?;

    // Send via curl
    let status = Command::new("curl")
        .arg("-X")
        .arg("POST")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(rendered)
        .arg(url)
        .arg("--silent")
        .arg("--output")
        .arg("/dev/null")
        .status()?;

    if !status.success() {
        println!(
            "⚠ Failed to send webhook notification (curl exit {})",
            status
        );
    }

    Ok(())
}

/// Get descriptions of currently passing features
fn get_passing_features(path: &Path) -> Result<std::collections::HashSet<String>> {
    if !path.exists() {
        return Ok(std::collections::HashSet::new());
    }
    let features = regression::parse_feature_list(path)?;
    Ok(features
        .into_iter()
        .filter(|f| f.passes)
        .map(|f| f.description)
        .collect())
}
