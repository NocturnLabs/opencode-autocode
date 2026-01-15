//! IPC server for communicating with the Go TUI client.
//!
//! The server spawns the Go process and communicates via stdin/stdout.

#![allow(dead_code)] // Some methods are reserved for future use

use super::protocol::*;
use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Name of the Go TUI binary.
const TUI_BINARY_NAME: &str = "opencode-forger-tui";

/// IPC server for managing communication with the Go TUI process.
pub struct IpcServer {
    child: Child,
    tx: Sender<Message>,
    rx: Receiver<Message>,
    debug: bool,
}

impl IpcServer {
    /// Spawn the Go TUI process and establish IPC channels.
    pub fn spawn() -> Result<Self> {
        Self::spawn_with_path(None)
    }

    /// Spawn the Go TUI process from a specific path.
    pub fn spawn_with_path(binary_path: Option<&Path>) -> Result<Self> {
        let debug = std::env::var("OPENCODE_RPC_DEBUG").is_ok_and(|v| v == "1");

        // Find the TUI binary
        let tui_path = if let Some(path) = binary_path {
            path.to_path_buf()
        } else {
            Self::find_tui_binary()?
        };

        if debug {
            eprintln!("[IPC DEBUG] Spawning TUI: {:?}", tui_path);
        }

        // Spawn the Go process
        let mut child = Command::new(&tui_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Let stderr pass through for debug output
            .spawn()
            .with_context(|| format!("Failed to spawn TUI binary: {:?}", tui_path))?;

        // Set up channels
        let (cmd_tx, cmd_rx) = mpsc::channel::<Message>();
        let (event_tx, event_rx) = mpsc::channel::<Message>();

        // Take ownership of stdin/stdout
        let stdin = child
            .stdin
            .take()
            .context("Failed to get stdin of TUI process")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to get stdout of TUI process")?;

        // Spawn writer thread (sends events to Go)
        let debug_writer = debug;
        thread::spawn(move || {
            let mut writer = stdin;
            while let Ok(msg) = event_rx.recv() {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if debug_writer {
                        eprintln!("[IPC DEBUG] OUT: {}", json);
                    }
                    if writeln!(writer, "{}", json).is_err() {
                        break;
                    }
                    if writer.flush().is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn reader thread (receives commands from Go)
        let debug_reader = debug;
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(json) => {
                        if debug_reader {
                            eprintln!("[IPC DEBUG] IN: {}", json);
                        }
                        match serde_json::from_str::<Message>(&json) {
                            Ok(msg) => {
                                if cmd_tx.send(msg).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                if debug_reader {
                                    eprintln!("[IPC DEBUG] Parse error: {}", e);
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            tx: event_tx,
            rx: cmd_rx,
            debug,
        })
    }

    /// Find the TUI binary in common locations.
    fn find_tui_binary() -> Result<std::path::PathBuf> {
        // Check relative to current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let candidate = exe_dir.join(TUI_BINARY_NAME);
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        // Check in target directory (for development)
        let target_dirs = ["target", "target/debug", "target/release"];
        for dir in target_dirs {
            let candidate = std::path::PathBuf::from(dir).join(TUI_BINARY_NAME);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Check in tui-go directory (for development)
        let tui_go_dirs = ["tui-go", "tui-go/target"];
        for dir in tui_go_dirs {
            let candidate = std::path::PathBuf::from(dir).join(TUI_BINARY_NAME);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // Check PATH
        if let Ok(output) = Command::new("which").arg(TUI_BINARY_NAME).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(std::path::PathBuf::from(path));
                }
            }
        }

        bail!(
            "Could not find TUI binary '{}'. Please ensure it is built and in your PATH.",
            TUI_BINARY_NAME
        )
    }

    /// Check if the TUI binary is available.
    pub fn is_available() -> bool {
        Self::find_tui_binary().is_ok()
    }

    /// Send an event to the Go TUI.
    pub fn send_event(&self, name: &str, payload: impl serde::Serialize) -> Result<()> {
        let msg = Message::event(name, payload)?;
        self.tx.send(msg).context("Failed to send event to TUI")?;
        Ok(())
    }

    /// Send an event without a payload.
    pub fn send_event_empty(&self, name: &str) -> Result<()> {
        let msg = Message::event_empty(name);
        self.tx.send(msg).context("Failed to send event to TUI")?;
        Ok(())
    }

    /// Receive a command from the Go TUI (blocking).
    pub fn recv_command(&self) -> Result<Message> {
        self.rx.recv().context("TUI process disconnected")
    }

    /// Try to receive a command without blocking.
    pub fn try_recv_command(&self) -> Option<Message> {
        self.rx.try_recv().ok()
    }

    /// Send the engine ready event.
    pub fn send_engine_ready(&self, version: &str, work_dir: &str) -> Result<()> {
        self.send_event(
            events::ENGINE_READY,
            EngineReadyPayload {
                version: version.to_string(),
                work_dir: work_dir.to_string(),
            },
        )
    }

    /// Send the mode list event.
    pub fn send_mode_list(&self, modes: Vec<ModeInfo>) -> Result<()> {
        self.send_event(events::MODE_LIST, ModeListPayload { modes })
    }

    /// Send config loaded status.
    pub fn send_config_loaded(&self, has_existing: bool, path: Option<&str>) -> Result<()> {
        self.send_event(
            events::CONFIG_LOADED,
            ConfigLoadedPayload {
                has_existing_config: has_existing,
                config_path: path.map(|s| s.to_string()),
            },
        )
    }

    /// Send a log line.
    pub fn send_log(&self, level: &str, message: &str) -> Result<()> {
        self.send_event(
            events::LOG_LINE,
            LogLinePayload {
                level: level.to_string(),
                message: message.to_string(),
                timestamp: Some(chrono::Local::now().to_rfc3339()),
            },
        )
    }

    /// Send a progress update.
    pub fn send_progress(
        &self,
        phase: &str,
        current: usize,
        total: usize,
        message: Option<&str>,
    ) -> Result<()> {
        let percentage = if total > 0 {
            Some((current as f64 / total as f64) * 100.0)
        } else {
            None
        };
        self.send_event(
            events::PROGRESS_UPDATE,
            ProgressUpdatePayload {
                phase: phase.to_string(),
                current,
                total,
                message: message.map(|s| s.to_string()),
                percentage,
            },
        )
    }

    /// Send a user prompt and wait for the response.
    pub fn prompt_select(
        &self,
        prompt_id: &str,
        message: &str,
        options: Vec<String>,
    ) -> Result<HandleSelectionPayload> {
        self.send_event(
            events::USER_PROMPT,
            UserPromptPayload {
                prompt_id: prompt_id.to_string(),
                prompt_type: "select".to_string(),
                message: message.to_string(),
                options: Some(options),
                default: None,
                allow_cancel: Some(true),
            },
        )?;

        // Wait for response
        loop {
            let msg = self.recv_command()?;
            if msg.name == commands::HANDLE_SELECTION {
                if let Some(payload) = msg.payload {
                    return serde_json::from_value(payload)
                        .context("Failed to parse selection response");
                }
            } else if msg.name == commands::CANCEL {
                bail!("User cancelled");
            }
        }
    }

    /// Send a completion event.
    pub fn send_finished(&self, success: bool, message: Option<&str>) -> Result<()> {
        self.send_event(
            events::FINISHED,
            FinishedPayload {
                success,
                message: message.map(|s| s.to_string()),
            },
        )
    }

    /// Send an error event.
    pub fn send_error(&self, message: &str, fatal: bool) -> Result<()> {
        self.send_event(
            events::ERROR,
            ErrorPayload {
                code: None,
                message: message.to_string(),
                fatal: Some(fatal),
            },
        )
    }

    /// Check if the TUI process is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Gracefully shut down the TUI process.
    pub fn shutdown(self) -> Result<()> {
        // Simply dropping self will trigger the Drop impl which kills the process
        // We just wait for it to finish here
        let mut server = self;

        // Wait for process to exit
        let status = server
            .child
            .wait()
            .context("Failed to wait for TUI process")?;

        if server.debug {
            eprintln!("[IPC DEBUG] TUI process exited with: {:?}", status);
        }

        // Prevent Drop from running
        std::mem::forget(server);

        Ok(())
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Try to kill the process if it's still running
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::event(
            events::ENGINE_READY,
            EngineReadyPayload {
                version: "0.10.0".to_string(),
                work_dir: "/tmp/test".to_string(),
            },
        )
        .unwrap();

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("rust->go"));
        assert!(json.contains("EngineReady"));
        assert!(json.contains("0.10.0"));
    }

    #[test]
    fn test_ipc_handshake_headless() {
        if !IpcServer::is_available() {
            eprintln!("Skipping test: TUI binary not found");
            return;
        }

        std::env::set_var("OPENCODE_TUI_HEADLESS", "1");
        let server = IpcServer::spawn().expect("Failed to spawn server");

        // Send ready
        server
            .send_engine_ready("0.10.0", "/tmp")
            .expect("Failed to send ready");

        // Send mode list
        server
            .send_mode_list(vec![ModeInfo {
                id: "generated".to_string(),
                label: "AI Generated".to_string(),
                description: "AI Generated mode".to_string(),
            }])
            .expect("Failed to send mode list");

        // Wait for selection (the headless client auto-selects the first mode)
        let msg = server.recv_command().expect("Failed to receive command");
        assert_eq!(msg.name, commands::SELECT_MODE);

        let payload: SelectModePayload = serde_json::from_value(msg.payload.unwrap()).unwrap();
        assert_eq!(payload.mode_id, "generated");

        // Send finished so headless client exits
        server
            .send_finished(true, Some("Test finished"))
            .expect("Failed to send finished");

        // Cleanup
        server.shutdown().expect("Failed to shutdown server");
        std::env::remove_var("OPENCODE_TUI_HEADLESS");
    }
}
