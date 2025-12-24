//! OpenCode session execution with timeout support

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

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
    println!("→ OpenCode exited with code: {}", status.code().unwrap_or(-1));

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
