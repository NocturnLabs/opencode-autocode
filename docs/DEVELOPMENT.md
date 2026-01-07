# Developer Onboarding & Development Guide

Welcome to the `opencode-autocode` developer guide. This document explains how to set up your environment and contribute to the core engine and templates.

## Prerequisites

- **Rust**: Latest stable (1.75+ recommended).
- **SQLite**: Included via `rusqlite` (bundled), but `sqlite3` CLI is useful for debugging.
- **OpenCode CLI**: Installed and configured with your provider of choice.

## Local Development Workflow

### 1. Initial Setup

```bash
git clone https://github.com/NocturnLabs/opencode-autocode.git
cd opencode-autocode
cargo build
```

### 2. Running the CLI locallly

Instead of installing, use `cargo run`:

```bash
# Preview scaffolding
cargo run -- --default --preview

# Run the TUI
cargo run -- --interactive
```

### 3. Testing
 
We use a comprehensive testing strategy:
 
- **Unit Tests**: Fast, internal logic (`cargo test`). Includes mocked `CommandRunner` tests.
- **Integration Tests**: File system and CLI entry points (`cargo test --test integration`).
- **Regression Tests**: Real-world feature verification (`make regression`).

To run the full suite:
```bash
cargo test --workspace
```

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
