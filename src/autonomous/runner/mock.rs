#![allow(dead_code)]
use super::traits::{CommandRunner, VerificationOutput};
use crate::autonomous::session::{SessionOptions, SessionResult};
use crate::common::logging::DebugLogger;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Mock command runner that returns pre-configured results
#[derive(Default)]
pub struct MockCommandRunner {
    /// Queue of session results to return
    pub session_results: Arc<Mutex<Vec<SessionResult>>>,
    /// Queue of verification results to return
    pub verification_results: Arc<Mutex<Vec<VerificationOutput>>>,
    /// Count of session executions
    pub session_call_count: Arc<Mutex<usize>>,
    /// Count of verification executions
    pub verification_call_count: Arc<Mutex<usize>>,
}

impl MockCommandRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a session result to the queue
    pub fn queue_session_result(&self, result: SessionResult) {
        self.session_results.lock().unwrap().push(result);
    }

    /// Add a verification result to the queue
    pub fn queue_verification_result(&self, result: VerificationOutput) {
        self.verification_results.lock().unwrap().push(result);
    }

    /// Get number of session calls made
    pub fn get_session_call_count(&self) -> usize {
        *self.session_call_count.lock().unwrap()
    }

    /// Get number of verification calls made
    pub fn get_verification_call_count(&self) -> usize {
        *self.verification_call_count.lock().unwrap()
    }
}

impl CommandRunner for MockCommandRunner {
    fn execute_session(
        &self,
        _options: SessionOptions,
        _logger: &DebugLogger,
    ) -> Result<SessionResult> {
        *self.session_call_count.lock().unwrap() += 1;

        let mut results = self.session_results.lock().unwrap();
        if results.is_empty() {
            Ok(SessionResult::Continue)
        } else {
            Ok(results.remove(0))
        }
    }

    fn run_verification(&self, _command: &str) -> Result<VerificationOutput> {
        *self.verification_call_count.lock().unwrap() += 1;

        let mut results = self.verification_results.lock().unwrap();
        if results.is_empty() {
            Ok(VerificationOutput {
                success: true,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            })
        } else {
            Ok(results.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomous::session::{SessionOptions, SessionResult};
    use crate::common::logging::DebugLogger;

    #[test]
    fn test_mock_runner_returns_queued_results() {
        let runner = MockCommandRunner::new();

        // Queue some results
        runner.queue_session_result(SessionResult::Continue);
        runner.queue_session_result(SessionResult::Error("test error".to_string()));

        runner.queue_verification_result(VerificationOutput {
            success: true,
            stdout: "All tests passed".to_string(),
            stderr: String::new(),
            exit_code: 0,
        });
        runner.queue_verification_result(VerificationOutput {
            success: false,
            stdout: String::new(),
            stderr: "Test failed".to_string(),
            exit_code: 1,
        });

        let logger = DebugLogger::new(false, None);

        // First session call
        let result1 = runner
            .execute_session(
                SessionOptions {
                    command: "cmd".to_string(),
                    model: "model".to_string(),
                    log_level: "info".to_string(),
                    session_id: None,
                    timeout_minutes: 0,
                    idle_timeout_seconds: 0,
                    opencode_path: "opencode".to_string(),
                },
                &logger,
            )
            .unwrap();
        assert!(matches!(result1, SessionResult::Continue));

        // Second session call
        let result2 = runner
            .execute_session(
                SessionOptions {
                    command: "cmd".to_string(),
                    model: "model".to_string(),
                    log_level: "info".to_string(),
                    session_id: None,
                    timeout_minutes: 0,
                    idle_timeout_seconds: 0,
                    opencode_path: "opencode".to_string(),
                },
                &logger,
            )
            .unwrap();
        assert!(matches!(result2, SessionResult::Error(_)));

        // First verification
        let v1 = runner.run_verification("test").unwrap();
        assert!(v1.success);
        assert_eq!(v1.stdout, "All tests passed");

        // Second verification
        let v2 = runner.run_verification("test").unwrap();
        assert!(!v2.success);
        assert_eq!(v2.exit_code, 1);

        // Check call counts
        assert_eq!(runner.get_session_call_count(), 2);
        assert_eq!(runner.get_verification_call_count(), 2);
    }

    #[test]
    fn test_mock_runner_defaults_when_queue_empty() {
        let runner = MockCommandRunner::new();
        let logger = DebugLogger::new(false, None);

        // Should return Continue by default
        let result = runner
            .execute_session(
                SessionOptions {
                    command: "cmd".to_string(),
                    model: "model".to_string(),
                    log_level: "info".to_string(),
                    session_id: None,
                    timeout_minutes: 0,
                    idle_timeout_seconds: 0,
                    opencode_path: "opencode".to_string(),
                },
                &logger,
            )
            .unwrap();
        assert!(matches!(result, SessionResult::Continue));

        // Should return success by default
        let v = runner.run_verification("test").unwrap();
        assert!(v.success);
    }
}
