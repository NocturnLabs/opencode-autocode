# Go TUI and IPC Protocol Documentation

This document describes the interactive TUI architecture and the JSON-RPC-style IPC protocol used for communication between the Rust engine and the Go Bubble Tea frontend.

## Architecture Overview

```
┌─────────────────────┐     stdin/stdout      ┌─────────────────────┐
│                     │◄──────────────────────►│                     │
│   Rust Engine       │      NDJSON IPC       │   Go Bubble Tea    │
│ (opencode-forger)   │◄──────────────────────►│   (opencode-forger │
│                     │                        │       -tui)        │
└─────────────────────┘                        └─────────────────────┘
         │                                              │
         ▼                                              ▼
   Business Logic                                 UI Rendering
   - Scaffolding                                  - Menus
   - AI Spec Generation                           - Progress
   - Database                                     - Prompts
   - Verification                                 - Logs
```

When `opencode-forger --interactive` is run:
1. The Rust CLI checks if the Go TUI binary is available
2. If found, it spawns the Go process and communicates via stdin/stdout
3. If not found, it falls back to the legacy Rust TUI (iocraft-based)

## IPC Protocol

### Message Format

All messages are **newline-delimited JSON (NDJSON)**. Each message is a single line of JSON followed by a newline character.

```json
{
  "protocol_version": "1.0.0",
  "direction": "rust->go" | "go->rust",
  "type": "event" | "command",
  "name": "<MessageName>",
  "payload": { ... }
}
```

### Protocol Version

Both sides check the `protocol_version` field. If there's a mismatch, the receiving side should display an error asking the user to update their binaries.

### Message Direction

- `rust->go`: Events sent from the Rust engine to the Go TUI
- `go->rust`: Commands sent from the Go TUI to the Rust engine

### Message Types

- `event`: Notifications that don't require a response (logs, progress, etc.)
- `command`: Requests that may trigger actions in the engine

## Events (Rust → Go)

### EngineReady

Sent when the Rust engine is ready to accept commands.

```json
{
  "name": "EngineReady",
  "payload": {
    "version": "0.10.0",
    "work_dir": "/path/to/project"
  }
}
```

### ConfigLoaded

Sent after checking for existing configuration.

```json
{
  "name": "ConfigLoaded",
  "payload": {
    "has_existing_config": true,
    "config_path": "/path/to/.forger/config.toml"
  }
}
```

### ModeList

Provides the available interactive modes for selection.

```json
{
  "name": "ModeList",
  "payload": {
    "modes": [
      {
        "id": "generated",
        "label": "🤖 AI Generated",
        "description": "Let AI create a full spec"
      },
      {
        "id": "manual",
        "label": "📝 Manual",
        "description": "Fill out project details"
      }
    ]
  }
}
```

### LogLine

A log message from the engine.

```json
{
  "name": "LogLine",
  "payload": {
    "level": "info",
    "message": "Processing feature...",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### ProgressUpdate

Progress update for long-running operations.

```json
{
  "name": "ProgressUpdate",
  "payload": {
    "phase": "scaffolding",
    "current": 3,
    "total": 10,
    "message": "Creating database schema...",
    "percentage": 30.0
  }
}
```

### UserPrompt

Request for user input or selection.

```json
{
  "name": "UserPrompt",
  "payload": {
    "prompt_id": "select_framework",
    "prompt_type": "select",
    "message": "Choose a framework:",
    "options": ["React", "Vue", "Svelte"],
    "default": "React",
    "allow_cancel": true
  }
}
```

### Finished

Signals that the engine has completed its work.

```json
{
  "name": "Finished",
  "payload": {
    "success": true,
    "message": "Project scaffolded successfully!"
  }
}
```

### Error

Reports an error from the engine.

```json
{
  "name": "Error",
  "payload": {
    "code": "SPEC_PARSE_ERROR",
    "message": "Failed to parse specification file",
    "fatal": true
  }
}
```

## Commands (Go → Rust)

### Confirm

Response to a confirmation prompt (e.g., setup choice).

```json
{
  "name": "Confirm",
  "payload": {
    "prompt_id": "setup_choice",
    "confirmed": true
  }
}
```

### SelectMode

User's mode selection.

```json
{
  "name": "SelectMode",
  "payload": {
    "mode_id": "generated"
  }
}
```

### HandleSelection

Response to a UserPrompt.

```json
{
  "name": "HandleSelection",
  "payload": {
    "prompt_id": "select_framework",
    "value": "React",
    "index": 0
  }
}
```

### Cancel

Request to cancel the current operation.

```json
{
  "name": "Cancel",
  "payload": {
    "reason": "User pressed Ctrl+C"
  }
}
```

### StartVibe

Initiate the vibe loop.

```json
{
  "name": "StartVibe",
  "payload": {
    "limit": 10,
    "single_model": false,
    "parallel": 4
  }
}
```

### Retry

Request to retry the last failed operation.

```json
{
  "name": "Retry",
  "payload": {}
}
```

## Building and Running

### Build Both Binaries

```bash
# Build everything
make build

# Build just Rust
make build-rust

# Build just Go TUI
make build-go
```

### Running with Debug Output

```bash
# Enable IPC debug logging
OPENCODE_RPC_DEBUG=1 ./target/release/opencode-forger --interactive
```

### Headless Mode (for Testing)

```bash
# Run the Go TUI in headless mode
OPENCODE_TUI_HEADLESS=1 ./target/release/opencode-forger-tui
```

## Extending the Protocol

1. Add new event/command names to `src/ipc/protocol.rs` (Rust) and `tui-go/internal/ipc/protocol.go` (Go)
2. Define payload structs in both languages
3. Handle the new messages in the appropriate locations:
   - Rust: `src/cli/handlers.rs` (sending) or event handlers
   - Go: `tui-go/internal/ui/model.go` (receiving and handling)
4. Increment `PROTOCOL_VERSION` if the change is breaking
