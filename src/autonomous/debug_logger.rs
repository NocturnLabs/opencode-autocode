//! Debug logger for developer mode
//!
//! Provides comprehensive logging to `opencode-debug.log` when developer mode is enabled.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::Mutex;

/// Debug log filename
pub const DEBUG_LOG_FILE: &str = "opencode-debug.log";

/// Thread-safe debug logger that writes timestamped entries to a log file
pub struct DebugLogger {
    writer: Option<Mutex<BufWriter<File>>>,
    enabled: bool,
}

impl DebugLogger {
    /// Create a new debug logger
    ///
    /// If `enabled` is true, creates/opens `opencode-debug.log` in the current directory.
    /// If `enabled` is false, all logging operations are no-ops.
    pub fn new(enabled: bool) -> Self {
        if !enabled {
            return Self {
                writer: None,
                enabled: false,
            };
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(DEBUG_LOG_FILE)
            .ok();

        Self {
            writer: file.map(|f| Mutex::new(BufWriter::new(f))),
            enabled: true,
        }
    }

    /// Log a message with the given level
    pub fn log(&self, level: &str, message: &str) {
        if !self.enabled {
            return;
        }

        if let Some(ref writer) = self.writer {
            if let Ok(mut w) = writer.lock() {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let _ = writeln!(w, "[{}] {} {}", timestamp, level, message);
                let _ = w.flush();
            }
        }
    }

    /// Log an INFO level message
    pub fn info(&self, message: &str) {
        self.log("INFO ", message);
    }

    /// Log a DEBUG level message
    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    /// Log an ERROR level message
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    /// Log a WARN level message
    
    pub fn warning(&self, message: &str) {
        self.log("WARN ", message);
    }

    /// Log a command being executed
    pub fn log_command(&self, program: &str, args: &[&str]) {
        if !self.enabled {
            return;
        }
        let cmd_str = format!("{} {}", program, args.join(" "));
        self.debug(&format!("Executing: {}", cmd_str));
    }

    /// Log subprocess output (stdout or stderr)
    pub fn log_output(&self, stream: &str, line: &str) {
        if !self.enabled {
            return;
        }
        self.debug(&format!("[{}] {}", stream, line));
    }

    /// Log a separator line for visual clarity
    pub fn separator(&self) {
        if !self.enabled {
            return;
        }
        self.log(
            "─────",
            "────────────────────────────────────────────────────",
        );
    }

    /// Check if logging is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Global logger instance for use across the autonomous module
///
/// This is set once at startup and accessed throughout the session.
static LOGGER: std::sync::OnceLock<DebugLogger> = std::sync::OnceLock::new();

/// Initialize the global logger
pub fn init(enabled: bool) {
    let _ = LOGGER.set(DebugLogger::new(enabled));
}

/// Get a reference to the global logger
pub fn get() -> &'static DebugLogger {
    LOGGER.get_or_init(|| DebugLogger::new(false))
}

/// Convenience macro for info logging
#[macro_export]
macro_rules! debug_info {
    ($($arg:tt)*) => {
        $crate::autonomous::debug_logger::get().info(&format!($($arg)*))
    };
}

/// Convenience macro for debug logging
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        $crate::autonomous::debug_logger::get().debug(&format!($($arg)*))
    };
}

/// Convenience macro for error logging
#[macro_export]
macro_rules! debug_error {
    ($($arg:tt)*) => {
        $crate::autonomous::debug_logger::get().error(&format!($($arg)*))
    };
}
