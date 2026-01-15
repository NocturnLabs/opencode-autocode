# Developer Onboarding & Development Guide

Welcome to the `opencode-forger` developer guide. This document explains how to set up your environment and contribute to the core engine and templates.

## Prerequisites

- **Rust**: Latest stable (1.75+ recommended).
- **Go**: Version 1.22+ (required for the interactive TUI).
- **SQLite**: Included via `rusqlite` (bundled), but `sqlite3` CLI is useful for debugging.
- **OpenCode CLI**: Installed and configured with your provider of choice.

## Local Development Workflow

### 1. Initial Setup

```bash
git clone https://github.com/NocturnLabs/opencode-forger.git
cd opencode-forger
make build
```

### 2. Running the CLI locally

The project consists of two main components: the Rust engine and the Go TUI.

```bash
# Build both
make build

# Run the TUI (it will use the Go binary in target/release/ if available)
./target/release/opencode-forger --interactive
```

For TUI development, you can run the Go binary independently for testing:
```bash
cd tui-go
go run ./cmd/opencode-forger-tui
```
*Note: Without the Rust engine on the other side of the pipe, you'll need to use `OPENCODE_TUI_HEADLESS=1` or just observe it waiting for input.*

### 3. Testing

We use a comprehensive testing strategy:

- **Unit Tests**: Fast, internal logic (`cargo test`).
- **Go Tests**: TUI logic and IPC protocol (`cd tui-go && go test ./...`).
- **Integration Tests**: File system and CLI entry points (`cargo test --test integration`).
- **Regression Tests**: Real-world feature verification (`make regression`).

To run the full suite:
```bash
make test
```

## Packaging & Releasing

OpenCode Forger is distributed as a pair of binaries.

### Release Artifacts
When building for release, both `opencode-forger` (Rust) and `opencode-forger-tui` (Go) must be included in the distribution package.

- **Linux/macOS**: Both binaries should be placed in the same directory (e.g., `/usr/local/bin` or `~/.cargo/bin`).
- **Archive Structure**:
  ```text
  opencode-forger-vX.Y.Z-OS-ARCH.tar.gz
  ├── opencode-forger      (Engine)
  └── opencode-forger-tui  (TUI Frontend)
  ```

### Build Command
Use the provided Makefile to build optimized binaries for both:
```bash
make build
```
This will place both binaries in `target/release/`.

### Cross-Compilation
The GitHub Actions workflow handles cross-compilation for Linux (x86_64) and macOS (x86_64, aarch64). If building manually:
- Use `cargo build --target ...` for Rust.
- Use `GOOS=... GOARCH=... go build` for Go.

## Working with Templates

Templates are located in `templates/`. If you modify a template, you must ensure it still parses correctly in `src/scaffold.rs`.

### Adding a new Project Type

1. Add the markdown template to `templates/projects/`.
2. Update the TUI selection logic in `src/tui/generated.rs` (if applicable) or `src/scaffold.rs`.

### Modifying "Progressive Discovery" Modules

Modules in `templates/modules/` are included into commands via `{{INCLUDE path}}`.

- Keep modules focused on a single responsibility (e.g., `testing.md`).
- Ensure identity and security headers are consistent across core modules.

## Project Structure at a Glance

```text
src/
├── autonomous/      # The Vibe Loop engine
├── conductor/       # Planning & Context
├── db/              # SQLite persistence
├── scaffold/        # Template expansion
└── tui/             # Interactive initialization
```

For a deeper dive into the architecture, see [ARCHITECTURE.md](../ARCHITECTURE.md).
