//! OpenCode session execution with timeout support

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use super::debug_logger::DebugLogger;

/// Result from a single OpenCode session
#[derive(Debug)]

pub enum SessionResult {
    /// Session completed successfully, continue to next
    Continue,
    /// All tests passing, project complete

    /// Error occurred, stop
    Error(String),
    /// Stop signal detected
    Stopped,
}

/// File checked for stop signal
pub const STOP_SIGNAL_FILE: &str = ".opencode-stop";

/// Polling interval for timeout checks (milliseconds)
const POLL_INTERVAL_MS: u64 = 500;

/// Execute an OpenCode session with optional timeout
pub fn execute_opencode_session(
    command: &str,
    model: &str,
    log_level: &str,
    session_id: Option<&str>,
    timeout_minutes: u32,
    idle_timeout_seconds: u32,
    logger: &DebugLogger,
) -> Result<SessionResult> {
    if stop_signal_exists() {
        logger.info("Stop signal detected before session start");
        return Ok(SessionResult::Stopped);
    }

    let mut cmd = build_opencode_command(command, model, log_level, session_id);
    logger.log_command(
        "opencode",
        &[
            "run",
            "--command",
            command,
            "--model",
            model,
            "--log-level",
            log_level,
        ],
    );

    execute_with_timeout(&mut cmd, timeout_minutes, idle_timeout_seconds, logger)
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

/// Patterns that indicate a feature was completed - trigger early termination
/// These are checked against stdout lines in real-time
const FEATURE_COMPLETE_PATTERNS: &[&str] = &[
    // Session complete signals
    "===SESSION_COMPLETE===",
    "SESSION_COMPLETE",
    // Git commit output (appears when commit succeeds)
    "[main ",   // git shows "[main abc1234] Commit message"
    "[master ", // for repos using master branch
    // Mark-pass output (backwards compat with old templates)
    "marked as passing",
    "Feature marked as passing",
    // Explicit completion markers the agent might output
    "Feature complete",
    "✅ Verified!",
];

/// Check if a line indicates feature completion
fn is_feature_complete_signal(line: &str) -> bool {
    FEATURE_COMPLETE_PATTERNS.iter().any(|p| line.contains(p))
}

fn execute_with_timeout(
    cmd: &mut Command,
    timeout_minutes: u32,
    idle_timeout_seconds: u32,
    logger: &DebugLogger,
) -> Result<SessionResult> {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timeout_secs = if timeout_minutes > 0 {
        timeout_minutes as u64 * 60
    } else {
        u64::MAX // Effectively unlimited
    };

    // Effective idle timeout (0 means disabled)
    let idle_enabled = idle_timeout_seconds > 0;

    println!("→ Session timeout: {} minutes", timeout_minutes);
    if idle_enabled {
        println!("→ Idle timeout: {} seconds", idle_timeout_seconds);
    }

    logger.debug(&format!(
        "Session timeout: {}m, Idle timeout: {}s",
        timeout_minutes, idle_timeout_seconds
    ));

    // Capture stdout and stderr
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn opencode command")?;
    let start = std::time::Instant::now();

    // Take ownership of stdout/stderr for reading
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Channel for signaling feature completion
    let (tx, rx) = mpsc::channel::<String>();
    let feature_completed = Arc::new(AtomicBool::new(false));

    // Last activity timestamp (Unix timestamp in seconds)
    // Initialized to now
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let last_activity = Arc::new(AtomicU64::new(now));

    // Spawn thread to read stdout with feature detection & activity tracking
    let tx_stdout = tx.clone();
    let feature_completed_stdout = Arc::clone(&feature_completed);
    let last_activity_stdout = Arc::clone(&last_activity);

    let stdout_handle = stdout.map(|s| {
        thread::spawn(move || {
            let reader = BufReader::new(s);
            let mut lines = Vec::new();
            for line in reader.lines().map_while(Result::ok) {
                // Update activity timestamp
                if idle_enabled {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    last_activity_stdout.store(now, Ordering::Relaxed);
                }

                println!("{}", line);

                // Check for feature completion signal
                if is_feature_complete_signal(&line)
                    && !feature_completed_stdout.load(Ordering::SeqCst)
                {
                    feature_completed_stdout.store(true, Ordering::SeqCst);
                    let _ = tx_stdout.send(line.clone());
                }

                lines.push(line);
            }
            lines
        })
    });

    // Spawn thread to read stderr with activity tracking
    let last_activity_stderr = Arc::clone(&last_activity);
    let stderr_handle = stderr.map(|s| {
        thread::spawn(move || {
            let reader = BufReader::new(s);
            let mut lines = Vec::new();
            for line in reader.lines().map_while(Result::ok) {
                // Update activity timestamp
                if idle_enabled {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    last_activity_stderr.store(now, Ordering::Relaxed);
                }

                eprintln!("{}", line);
                lines.push(line);
            }
            lines
        })
    });

    // Track if we terminated early due to feature completion
    let mut terminated_for_isolation = false;

    loop {
        // Check for feature completion signal (non-blocking)
        if let Ok(trigger_line) = rx.try_recv() {
            println!();
            println!("🛑 ISOLATION: Feature completed, terminating session early");
            println!("   Trigger: {}", trigger_line);
            logger.info(&format!(
                "Isolation enforcement: terminating after feature completion. Trigger: {}",
                trigger_line
            ));

            // Give the session a moment to finish any pending writes
            thread::sleep(Duration::from_millis(1000));
            terminate_child(&mut child);
            terminated_for_isolation = true;
            break;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished naturally - handle output collection ...
                // Reusing exit logic below loop for cleaner flow

                // Wait for output threads to finish
                if let Some(handle) = stdout_handle {
                    if let Ok(lines) = handle.join() {
                        if logger.is_enabled() {
                            for line in lines {
                                logger.log_output("stdout", &line);
                            }
                        }
                    }
                }
                if let Some(handle) = stderr_handle {
                    if let Ok(lines) = handle.join() {
                        if logger.is_enabled() {
                            for line in lines {
                                logger.log_output("stderr", &line);
                            }
                        }
                    }
                }

                println!();
                let exit_code = status.code().unwrap_or(-1);
                println!("→ OpenCode exited with code: {}", exit_code);
                logger.info(&format!("OpenCode exited with code: {}", exit_code));

                if !status.success() {
                    let err_msg = format!("exit code {}", exit_code);
                    logger.error(&format!("Session failed: {}", err_msg));
                    return Ok(SessionResult::Error(err_msg));
                }
                break;
            }
            Ok(None) => {
                // 1. Hard Timeout Check
                if start.elapsed().as_secs() > timeout_secs {
                    println!();
                    println!("⏱ Session timeout reached ({} minutes)", timeout_minutes);
                    logger.error(&format!(
                        "Session timeout after {} minutes",
                        timeout_minutes
                    ));
                    terminate_child(&mut child);
                    return Ok(SessionResult::Error("session timeout".to_string()));
                }

                // 2. Idle Timeout Check
                if idle_enabled {
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let last = last_activity.load(Ordering::Relaxed);
                    if current_time > last + idle_timeout_seconds as u64 {
                        println!();
                        println!(
                            "💤 Idle timeout reached (no output for {} seconds)",
                            idle_timeout_seconds
                        );
                        logger.error(&format!(
                            "Idle timeout after {} seconds of silence",
                            idle_timeout_seconds
                        ));
                        terminate_child(&mut child);
                        return Ok(SessionResult::Error("idle timeout".to_string()));
                    }
                }

                if stop_signal_exists() {
                    println!();
                    println!("→ Stop signal detected, terminating session...");
                    logger.info("Stop signal detected, terminating session");
                    terminate_child(&mut child);
                    return Ok(SessionResult::Stopped);
                }

                thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
            }
            Err(e) => {
                logger.error(&format!("Failed to check process status: {}", e));
                return Err(e).context("Failed to check process status");
            }
        }
    }

    // If we terminated for isolation, still return Continue so supervisor can verify
    if terminated_for_isolation {
        return Ok(SessionResult::Continue);
    }

    if stop_signal_exists() {
        logger.info("Stop signal detected after session");
        return Ok(SessionResult::Stopped);
    }

    Ok(SessionResult::Continue)
}

fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}
