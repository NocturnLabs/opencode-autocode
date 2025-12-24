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
pub fn run(limit: Option<usize>, config_path: Option<&Path>, developer_mode: bool) -> Result<()> {
    // Set up developer mode logging
    let log_file = if developer_mode {
        let log_path = Path::new("opencode-debug.log");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .context("Failed to create debug log file")?;
        println!("🔧 Developer mode enabled - logging to: {}", log_path.display());
        Some(std::sync::Mutex::new(file))
    } else {
        None
    };

    // Helper macro for logging
    macro_rules! dev_log {
        ($($arg:tt)*) => {
            if let Some(ref file_mutex) = log_file {
                use std::io::Write;
                if let Ok(mut file) = file_mutex.lock() {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    let _ = writeln!(file, "[{}] {}", timestamp, format!($($arg)*));
                    let _ = file.flush();
                }
            }
        };
    }

    dev_log!("=== OpenCode Autocode Developer Log ===");
    dev_log!("Started at: {}", chrono::Local::now());
    dev_log!("Working directory: {:?}", std::env::current_dir());
    dev_log!("Rust version: {}", env!("CARGO_PKG_VERSION"));
    dev_log!("");

    // Load configuration
    dev_log!("Loading configuration...");
    let config = match config_path {
        Some(path) => {
            dev_log!("  Config path: {:?}", path);
            Config::load_from_file(path)?
        }
        None => {
            dev_log!("  Using default config path (autocode.toml)");
            Config::load(None)?
        }
    };

    // Log full configuration
    dev_log!("Configuration loaded:");
    dev_log!("  models.autonomous: {}", config.models.autonomous);
    dev_log!("  models.default: {}", config.models.default);
    dev_log!("  models.reasoning: {}", config.models.reasoning);
    dev_log!("  models.enhancement: {}", config.models.enhancement);
    dev_log!("  autonomous.max_iterations: {}", config.autonomous.max_iterations);
    dev_log!("  autonomous.delay_between_sessions: {}", config.autonomous.delay_between_sessions);
    dev_log!("  autonomous.log_level: {}", config.autonomous.log_level);
    dev_log!("  agent.max_retry_attempts: {}", config.agent.max_retry_attempts);
    dev_log!("  paths.feature_list_file: {}", config.paths.feature_list_file);
    dev_log!("  notifications.webhook_enabled: {}", config.notifications.webhook_enabled);
    dev_log!("");

    let delay = config.autonomous.delay_between_sessions;
    let max_iterations = if config.autonomous.max_iterations == 0 {
        limit.unwrap_or(usize::MAX)
    } else {
        limit.unwrap_or(config.autonomous.max_iterations as usize)
    };
    let model = &config.models.autonomous;
    let log_level = &config.autonomous.log_level;
    let feature_list_file = &config.paths.feature_list_file;
    let session_timeout = config.autonomous.session_timeout_minutes;
    let auto_commit = config.autonomous.auto_commit;
    let verbose = config.ui.verbose;

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
    if developer_mode {
        println!("Developer mode: ENABLED (see opencode-debug.log)");
    }
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
        dev_log!("=== Session {} started ===", iteration);

        // Check max iterations
        if iteration > max_iterations {
            dev_log!("Reached max iterations ({}), stopping", max_iterations);
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
        dev_log!("Feature list path: {:?}", feature_path);
        dev_log!("Feature list exists: {}", feature_path.exists());

        let command = if !feature_path.exists() {
            dev_log!("First run detected, using auto-init");
            println!("→ First run: auto-init");
            "auto-init"
        } else {
            // Count remaining tests
            let (passing, remaining) = count_feature_status(feature_path)?;
            dev_log!("Feature status: {} passing, {} remaining", passing, remaining);
            println!("→ Progress: {} passing, {} remaining", passing, remaining);

            if remaining == 0 && passing > 0 {
                dev_log!("All tests passing! Project complete!");
                println!();
                println!("🎉 All tests passing! Project complete!");
                break;
            }

            "auto-continue"
        };

        dev_log!("Running command: {}", command);
        println!("→ Running: opencode run --command /{}", command);
        println!();

        // Capture passing features before session
        let before_passing = get_passing_features(feature_path)?;
        dev_log!("Features passing before session: {:?}", before_passing);

        // Execute opencode
        dev_log!("Executing opencode session...");
        dev_log!("  Model: {}", model);
        dev_log!("  Log level: {}", log_level);
        dev_log!("  Session timeout: {} minutes", session_timeout);
        let result = run_opencode_session(command, model, log_level, session_id.as_deref(), session_timeout, developer_mode)?;
        dev_log!("Session result: {:?}", result);

        // Capture passing features after session
        let after_passing = get_passing_features(feature_path)?;
        dev_log!("Features passing after session: {:?}", after_passing);

        // Find newly completed features
        let new_features: Vec<_> = after_passing.difference(&before_passing).collect();
        dev_log!("Newly completed features: {:?}", new_features);

        // Get current progress for webhook
        let (current_passing, total_remaining) = if feature_path.exists() {
            count_feature_status(feature_path).unwrap_or((0, 0))
        } else {
            (0, 0)
        };
        let total_features = current_passing + total_remaining;
        
        for feature_desc in new_features {
            // Find the full feature object
            if let Ok(features) = regression::parse_feature_list(feature_path) {
                if let Some(feature) = features
                    .into_iter()
                    .find(|f| f.description == *feature_desc)
                {
                    dev_log!("Sending webhook for feature: {}", feature_desc);
                    if let Err(e) = send_webhook_notification(
                        &config,
                        &feature,
                        iteration,
                        current_passing,
                        total_features,
                    ) {
                        dev_log!("Webhook error: {}", e);
                        println!("⚠ Webhook error: {}", e);
                    }

                    // Auto-commit if enabled
                    if auto_commit {
                        dev_log!("Auto-committing feature: {}", feature_desc);
                        if let Err(e) = auto_commit_feature(&feature.description, verbose) {
                            dev_log!("Auto-commit error: {}", e);
                            if verbose {
                                println!("⚠ Auto-commit error: {}", e);
                            }
                        }
                    }
                }
            }
        }

        match result {
            SessionResult::Continue => {
                consecutive_errors = 0; // Reset on success
                dev_log!("Session completed successfully, continuing...");
                println!("→ Session complete, continuing...");
                println!("→ Next session in {}s (Ctrl+C to stop)", delay);
                thread::sleep(Duration::from_secs(delay as u64));
            }
            SessionResult::Complete => {
                dev_log!("PROJECT COMPLETE signal received");
                println!();
                println!("🎉 All tests passing! Project complete!");
                break;
            }
            SessionResult::Error(msg) => {
                consecutive_errors += 1;
                dev_log!("Session error (attempt {}/{}): {}", consecutive_errors, max_retries, msg);
                println!();
                println!(
                    "⚠ Session error (attempt {}/{}): {}",
                    consecutive_errors, max_retries, msg
                );

                if consecutive_errors >= max_retries {
                    dev_log!("Exceeded max retries, stopping");
                    println!("❌ Exceeded max retries ({}), stopping.", max_retries);
                    break;
                }

                // Exponential backoff: delay * 2^(attempts-1)
                let backoff = delay * (1 << (consecutive_errors - 1).min(4));
                dev_log!("Exponential backoff: {}s", backoff);
                println!("→ Retrying in {}s (exponential backoff)...", backoff);
                thread::sleep(Duration::from_secs(backoff as u64));
            }
            SessionResult::Stopped => {
                dev_log!("Stop signal detected (.opencode-stop file exists)");
                println!();
                println!("Stop signal detected (.opencode-stop file exists)");
                let _ = std::fs::remove_file(".opencode-stop");
                break;
            }
        }
    }

    dev_log!("=== Runner stopped ===");
    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  Runner stopped");
    println!("═══════════════════════════════════════════════════");
    println!();

    // Print final status
    let feature_path = Path::new(feature_list_file);
    if feature_path.exists() {
        let (passing, remaining) = count_feature_status(feature_path)?;
        dev_log!("Final status: {} / {} tests passing", passing, passing + remaining);
        println!(
            "Status: {} / {} tests passing",
            passing,
            passing + remaining
        );
    }

    if developer_mode {
        println!();
        println!("📋 Debug log saved to: opencode-debug.log");
        dev_log!("=== Log complete ===");
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

/// Run a single OpenCode session with optional timeout
fn run_opencode_session(
    command: &str,
    model: &str,
    log_level: &str,
    session_id: Option<&str>,
    timeout_minutes: u32,
    _developer_mode: bool,
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

    // If timeout is configured, spawn with timeout handling
    if timeout_minutes > 0 {
        let timeout_secs = timeout_minutes as u64 * 60;
        println!("→ Session timeout: {} minutes", timeout_minutes);

        let mut child = cmd.spawn().context("Failed to spawn opencode command")?;

        // Wait with timeout
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished
                    println!();
                    println!("→ OpenCode exited with code: {}", status.code().unwrap_or(-1));

                    if !status.success() {
                        return Ok(SessionResult::Error(format!(
                            "exit code {}",
                            status.code().unwrap_or(-1)
                        )));
                    }
                    break;
                }
                Ok(None) => {
                    // Still running, check timeout
                    if start.elapsed().as_secs() > timeout_secs {
                        println!();
                        println!("⏱ Session timeout reached ({} minutes), terminating...", timeout_minutes);
                        let _ = child.kill();
                        let _ = child.wait(); // Reap the process
                        return Ok(SessionResult::Error("session timeout".to_string()));
                    }
                    // Check for stop signal while running
                    if Path::new(".opencode-stop").exists() {
                        println!();
                        println!("→ Stop signal detected, terminating session...");
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok(SessionResult::Stopped);
                    }
                    // Sleep briefly before checking again
                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => {
                    return Err(e).context("Failed to check process status");
                }
            }
        }
    } else {
        // No timeout, run synchronously
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
    }

    // Check for stop signal after running
    if Path::new(".opencode-stop").exists() {
        return Ok(SessionResult::Stopped);
    }

    Ok(SessionResult::Continue)
}

/// Send a webhook notification for a completed feature
fn send_webhook_notification(
    config: &Config,
    feature: &regression::Feature,
    session_number: usize,
    current_passing: usize,
    total_features: usize,
) -> Result<()> {
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

    // Format verification steps (limit to first 5 for embed)
    let verification_steps = if feature.steps.is_empty() {
        "No steps defined".to_string()
    } else {
        feature.steps
            .iter()
            .take(5)
            .map(|s| format!("• {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Calculate progress percentage
    let progress_percent = if total_features > 0 {
        (current_passing * 100) / total_features
    } else {
        0
    };

    // Prepare template data
    let mut data = std::collections::HashMap::new();
    data.insert("feature_name", feature.description.clone());
    data.insert("feature_category", capitalize_first(&feature.category));
    data.insert("project_name", project_name);
    data.insert("timestamp", timestamp);
    data.insert("session_number", session_number.to_string());
    data.insert("progress_current", current_passing.to_string());
    data.insert("progress_total", total_features.to_string());
    data.insert("progress_percent", progress_percent.to_string());
    data.insert("verification_steps", verification_steps);

    // Render template
    let handlebars = handlebars::Handlebars::new();
    let template_path = Path::new("templates/notifications/webhook.json");
    let template_content = if template_path.exists() {
        std::fs::read_to_string(template_path)?
    } else {
        // Fallback minimal template if file is missing
        r#"{ "content": "✅ Feature Completed: {{feature_name}} in {{project_name}} ({{progress_current}}/{{progress_total}})" }"#.to_string()
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

/// Auto-commit a completed feature to git
fn auto_commit_feature(feature_description: &str, verbose: bool) -> Result<()> {
    // Stage all changes
    let add_status = Command::new("git")
        .args(["add", "."])
        .status()
        .context("Failed to run git add")?;

    if !add_status.success() {
        anyhow::bail!("git add failed with exit code {}", add_status.code().unwrap_or(-1));
    }

    // Create commit with feature description
    let commit_msg = format!("feat: {}", feature_description);
    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .status()
        .context("Failed to run git commit")?;

    if commit_status.success() {
        if verbose {
            println!("✓ Auto-committed: {}", commit_msg);
        }
    } else {
        // Exit code 1 often means "nothing to commit" which is OK
        if verbose {
            println!("→ Git commit skipped (nothing to commit or already committed)");
        }
    }

    Ok(())
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
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
