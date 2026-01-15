# OpenCode Forger TUI

A Bubble Tea-based terminal user interface for OpenCode Forger.

## Overview

This Go module provides the interactive TUI frontend for OpenCode Forger. It communicates with the Rust engine via JSON-RPC-style messages over stdin/stdout, allowing the UI to be developed and tested independently from the core engine logic.

## Building

```bash
# Build the TUI binary
make build

# Or manually
go build -o ../target/release/opencode-forger-tui ./cmd/opencode-forger-tui
```

## Running

The TUI is normally launched automatically by the main `opencode-forger` binary when running in interactive mode:

```bash
opencode-forger --interactive
```

For standalone testing or debugging:

```bash
# Run with IPC debug logging
OPENCODE_RPC_DEBUG=1 ./opencode-forger-tui

# Run in headless mode (for integration tests)
OPENCODE_TUI_HEADLESS=1 ./opencode-forger-tui
```

## Project Structure

```
tui-go/
├── cmd/
│   └── opencode-forger-tui/
│       └── main.go           # Entry point
├── internal/
│   ├── ipc/
│   │   ├── client.go         # IPC communication client
│   │   └── protocol.go       # Message types and constants
│   └── ui/
│       ├── model.go          # Main Bubble Tea model
│       ├── styles.go         # Theme and styling
│       └── views.go          # View rendering methods
├── go.mod
├── go.sum
└── Makefile
```

## IPC Protocol

See [IPC_PROTOCOL.md](../docs/IPC_PROTOCOL.md) for the full protocol specification.

### Quick Reference

**Events (Rust → Go):**
- `EngineReady` - Engine is ready
- `ConfigLoaded` - Config status
- `ModeList` - Available modes
- `LogLine` - Log message
- `ProgressUpdate` - Progress info
- `UserPrompt` - User input request
- `Finished` - Operation complete
- `Error` - Error occurred

**Commands (Go → Rust):**
- `Confirm` - Confirmation response
- `SelectMode` - Mode selection
- `HandleSelection` - Prompt response
- `Cancel` - Cancel request
- `StartVibe` - Start vibe loop
- `Retry` - Retry last operation

## Development

### Adding New Views

1. Create a new view method in `internal/ui/views.go`
2. Add a new phase constant in `internal/ui/model.go`
3. Handle phase transitions in the `Update` method
4. Wire up the view in the `View` method

### Adding New IPC Messages

1. Add constants in `internal/ipc/protocol.go`
2. Define payload structs
3. Handle in `model.go`'s `handleIpcMessage` method

## Testing

```bash
make test
```

## License

MIT
