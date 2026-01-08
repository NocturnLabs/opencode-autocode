#![allow(dead_code)]
use crate::autonomous::session::{SessionOptions, SessionResult};
use crate::common::logging::DebugLogger;
use anyhow::Result;

/// Output from a verification command
#[derive(Debug, Clone)]
pub struct VerificationOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Trait for executing commands, enabling dependency injection for testing
pub trait CommandRunner: Send + Sync {
    /// Execute an OpenCode session
    fn execute_session(
        &self,
        options: SessionOptions,
        logger: &DebugLogger,
    ) -> Result<SessionResult>;

    /// Run a verification command
    fn run_verification(&self, command: &str) -> Result<VerificationOutput>;
}
