//! OpenCode session execution with timeout support

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Token usage statistics from OpenCode
#[derive(Debug, Default, Clone)]
pub struct TokenStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
}

/// Fetch token stats for the current project by running `opencode stats`
pub fn fetch_token_stats() -> Option<TokenStats> {
    let output = Command::new("opencode")
        .args(["stats", "--project", ""])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_token_stats(&stdout)
}

fn parse_token_stats(output: &str) -> Option<TokenStats> {
    let mut stats = TokenStats::default();

    for line in output.lines() {
        let line = line.trim();

        // Parse lines like "Input tokens: 123,456" or "Total input: 123456"
        if line.to_lowercase().contains("input") && line.contains("token") {
            if let Some(num) = extract_number(line) {
                stats.input_tokens = num;
            }
        }

        // Parse lines like "Output tokens: 78,901"
        if line.to_lowercase().contains("output") && line.contains("token") {
            if let Some(num) = extract_number(line) {
                stats.output_tokens = num;
            }
        }

        // Parse cost lines like "Total cost: $1.23" or "Cost: $0.05"
        if line.to_lowercase().contains("cost") {
            if let Some(cost) = extract_cost(line) {
                stats.total_cost = cost;
            }
        }
    }

    if stats.input_tokens > 0 || stats.output_tokens > 0 {
        Some(stats)
    } else {
        None
    }
}

fn extract_number(s: &str) -> Option<u64> {
    // Find sequences of digits, removing commas
    let num_str: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',')
        .collect::<String>()
        .replace(',', "");

    // Take the last number in the string (usually the value after the label)
    num_str.split_whitespace().last()?.parse().ok()
        .or_else(|| num_str.parse().ok())
}

fn extract_cost(s: &str) -> Option<f64> {
    // Find pattern like $1.23 or 1.23
    for part in s.split_whitespace() {
        let cleaned = part.trim_start_matches('$').trim_end_matches(',');
        if let Ok(cost) = cleaned.parse::<f64>() {
            return Some(cost);
        }
    }
    None
}

/// Result from a single OpenCode session
#[derive(Debug)]
#[allow(dead_code)]
pub enum SessionResult {
    /// Session completed successfully, continue to next
    Continue,
    /// All tests passing, project complete
    Complete,
    /// Error occurred, stop
    Error(String),
    /// Stop signal detected
    Stopped,
}

/// File checked for stop signal
const STOP_SIGNAL_FILE: &str = ".opencode-stop";

/// Polling interval for timeout checks (milliseconds)
const POLL_INTERVAL_MS: u64 = 500;

/// Execute an OpenCode session with optional timeout
pub fn execute_opencode_session(
    command: &str,
    model: &str,
    log_level: &str,
    session_id: Option<&str>,
    timeout_minutes: u32,
) -> Result<SessionResult> {
    if stop_signal_exists() {
        return Ok(SessionResult::Stopped);
    }

    let mut cmd = build_opencode_command(command, model, log_level, session_id);

    if timeout_minutes > 0 {
        execute_with_timeout(&mut cmd, timeout_minutes)
    } else {
        execute_synchronously(&mut cmd)
    }
}

/// Check if stop signal file exists
pub fn stop_signal_exists() -> bool {
    Path::new(STOP_SIGNAL_FILE).exists()
}

/// Remove the stop signal file
pub fn clear_stop_signal() {
    let _ = std::fs::remove_file(STOP_SIGNAL_FILE);
}

fn build_opencode_command(
    command: &str,
    model: &str,
    log_level: &str,
    session_id: Option<&str>,
) -> Command {
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

    cmd
}

fn execute_with_timeout(cmd: &mut Command, timeout_minutes: u32) -> Result<SessionResult> {
    let timeout_secs = timeout_minutes as u64 * 60;
    println!("→ Session timeout: {} minutes", timeout_minutes);

    let mut child = cmd.spawn().context("Failed to spawn opencode command")?;
    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
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
                break;
            }
            Ok(None) => {
                if start.elapsed().as_secs() > timeout_secs {
                    println!();
                    println!("⏱ Session timeout reached ({} minutes)", timeout_minutes);
                    terminate_child(&mut child);
                    return Ok(SessionResult::Error("session timeout".to_string()));
                }

                if stop_signal_exists() {
                    println!();
                    println!("→ Stop signal detected, terminating session...");
                    terminate_child(&mut child);
                    return Ok(SessionResult::Stopped);
                }

                thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
            }
            Err(e) => {
                return Err(e).context("Failed to check process status");
            }
        }
    }

    if stop_signal_exists() {
        return Ok(SessionResult::Stopped);
    }

    Ok(SessionResult::Continue)
}

fn execute_synchronously(cmd: &mut Command) -> Result<SessionResult> {
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

    if stop_signal_exists() {
        return Ok(SessionResult::Stopped);
    }

    Ok(SessionResult::Continue)
}

fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}
