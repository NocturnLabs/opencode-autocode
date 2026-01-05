/// Types of verification failures - determines corrective action
#[derive(Debug, PartialEq)]
pub enum VerificationFailure {
    /// Filter/grep didn't match any tests (command is wrong, not code)
    NoTestsMatch,
    /// Test file doesn't exist
    TestFileMissing,
    /// Command itself is invalid (missing binary, syntax error)
    CommandError,
    /// Actual test assertion failure (real regression)
    AssertionFailure,
}

impl VerificationFailure {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoTestsMatch => "no tests matched filter",
            Self::TestFileMissing => "test file missing",
            Self::CommandError => "command error",
            Self::AssertionFailure => "assertion failure",
        }
    }
}

/// Classify a verification failure based on error output
pub fn classify_verification_failure(error: &str) -> VerificationFailure {
    let lower = error.to_lowercase();

    // Patterns that indicate the verification command is broken, not the code
    if lower.contains("no test files")
        || lower.contains("did not match any")
        || lower.contains("filters did not match")
        || lower.contains("pattern not found")
        || lower.contains("no tests found")
        || lower.contains("no specs found")
    {
        return VerificationFailure::NoTestsMatch;
    }

    if lower.contains("cannot find")
        || lower.contains("cannot find package")
        || lower.contains("no such file")
        || lower.contains("file not found")
        || lower.contains("enoent")
    {
        // Distinguish between missing test files and missing dependencies
        if lower.contains("package") || lower.contains("module") {
            return VerificationFailure::CommandError;
        }
        return VerificationFailure::TestFileMissing;
    }

    if lower.contains("command not found")
        || lower.contains("unknown command")
        || lower.contains("not recognized")
        || lower.contains("spawn unknown")
        || lower.contains("permission denied")
    {
        return VerificationFailure::CommandError;
    }

    // Default: assume actual test failure (code issue)
    VerificationFailure::AssertionFailure
}
