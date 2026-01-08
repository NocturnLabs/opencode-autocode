#![allow(dead_code)]
use super::traits::{CommandRunner, VerificationOutput};
use crate::autonomous::session::{self, SessionOptions, SessionResult};
use crate::common::logging::DebugLogger;
use anyhow::Result;

/// Real implementation that executes actual commands
#[derive(Default)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn execute_session(
        &self,
        options: SessionOptions,
        logger: &DebugLogger,
    ) -> Result<SessionResult> {
        session::execute_opencode_session(options, logger)
    }

    fn run_verification(&self, command: &str) -> Result<VerificationOutput> {
        use std::process::Command;

        // SECURITY NOTE: This uses `sh -c` which is shell-injection-prone if untrusted
        // input reaches this function. Currently, verification commands come from
        // forger.toml which is developer-controlled configuration. If this changes
        // in the future (e.g., user-provided verification commands), this should be
        // refactored to use structured {program, args[]} format or command validation.
        let output = Command::new("sh").arg("-c").arg(command).output()?;

        Ok(VerificationOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }
}
