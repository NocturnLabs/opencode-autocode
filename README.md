# OpenCode Autocode

A Rust CLI that scaffolds autonomous coding projects for [OpenCode](https://github.com/sst/opencode) and runs them to completion. It bridges the gap between high-level application specs and fully implemented features.

> [!WARNING]
> **AI-Generated Code Disclaimer**: Significant portions of this codebase (including logic, templates, and tests) were generated or refined using Large Language Models. Use with appropriate caution and always review changes in your local projects.

![OpenCode Autocode Demo](assets/demo.gif)

## Documentation

- [Architecture Overview](ARCHITECTURE.md) - High-level system design and data flow.
- [Development Guide](docs/DEVELOPMENT.md) - Developer onboarding and contribution guidelines.
- [Contributing](CONTRIBUTING.md) - Code standards and PR process.

## Quick Start

```bash
# 1. Scaffold a new project (Interactive TUI)
opencode-autocode --interactive

# 2. Configure project settings
opencode-autocode --config

# 3. Start the autonomous vibe loop
opencode-autocode vibe --developer
```

## Features

- 🚀 **Zero-Config Scaffolding**: Build complex app specs using a rich interactive TUI, now stored in `.autocode/`.
- 🗃️ **SQLite Persistence**: Industrial-grade progress tracking with a relational database (`.autocode/progress.db`).
- 🔄 **Vibe Loop**: Automated session management with intelligent continuation and exponential backoff retry logic.
- 🧠 **Conductor Workflow**: Context-driven planning that creates persistent `.conductor/` and `tracks/` directories to maintain project state.
- 📝 **Developer Mode**: Detailed output captured in `opencode-debug.log` for debugging autonomous sessions.
- ✅ **Auto-Commit**: Automatically commits each completed feature to Git with AI-generated messages.
- 🔁 **Stuck Recovery**: Automatically generates alternative implementation paths when progress stalls.
- 🧪 **Regression Testing**: CLI command to verify all previously completed features directly from the database.
- 🔔 **Webhooks**: Real-time integration with Discord/Slack for feature completion alerts.
- 🛠️ **MCP Native**: First-class support for Model Context Protocol tools like `osgrep`, `chrome-devtools`, and `sequential-thinking`.
- 🔌 **Port Conflict Prevention**: Automatic detection and resolution of port conflicts before starting servers or tests.
- 📦 **Module Verification**: Validates ES6 import/export consistency to prevent ReferenceErrors at runtime.
- 🧩 **Progressive Discovery**: Modular template system that reduces context window usage by ~80%.

## CLI Reference

### Scaffolding Mode

- `--interactive` (alias: `--init`): Start the interactive spec-building TUI.
- `--default`: Scaffold using the built-in default template immediately.
- `--spec <FILE>`: Use a custom markdown specification file.
- `--output <DIR>` (alias: `-o`): Specify the target directory for scaffolding.
- `--preview` (alias: `--dry-run`): Preview what will be created without writing to disk.

### Vibe Mode (Autonomous Loop)

- `vibe`: Start the main session loop.
  - `--developer`: Enable verbose debug logging to file.
  - `--limit <N>`: Stop the loop after N iterations.
  - `--config-file <FILE>`: Load a custom TOML configuration.

### Database Operations

- `db init`: Initialize a new progress database.
- `db migrate`: Import legacy `feature_list.json` data into the SQLite database.
- `db stats`: View high-level feature and session statistics.
- `db export <FILE>`: Export the database contents to a JSON file.
- `db query "<SQL>"`: Execute a SELECT query and display results.
- `db exec "<SQL>"`: Execute a write query (INSERT/UPDATE/DELETE).
- `db tables`: List all tables in the database.
- `db schema <table>`: Show the schema for a specific table.
- `db next-feature`: Get the next incomplete feature.
- `db mark-pass <id>`: Mark a feature as passing.

### Utility Commands

- `--config`: Launch the settings configuration TUI.
- `--regression-check`: Verify all features marked as passing in the database.
- `templates list`: View available project templates (Web App, CLI, API).
- `templates use <name>`: Scaffold a project directly from a named template.

## Configuration

Settings are stored in `.autocode/config.toml`. You can either use `opencode-autocode --config` or edit the file manually. Paths support environment variables like `$HOME`.

```toml
[models]
default = "opencode/big-pickle"    # Used for spec generation
autonomous = "opencode/grok-code" # Used for code implementation
reasoning = "opencode/grok-code"  # Used for planning and logic
enhancement = "opencode/big-pickle" # Used for discover_improvements

[paths]
database_file = ".autocode/progress.db"
log_dir = "$HOME/.local/share/opencode/log"

[autonomous]
delay_between_sessions = 5      # Seconds to wait
max_iterations = 0              # 0 = Run until complete
session_timeout_minutes = 60    # Kill hung sessions after N minutes
auto_commit = true              # Commit to Git on feature completion
log_level = "DEBUG"             # Logging verbosity

[agent]
max_retry_attempts = 3          # Attempts before switching to research mode
max_research_attempts = 3       # Attempts before giving up
single_feature_focus = true     # Focus AI on one feature at a time

[alternative_approaches]
enabled = true                  # Enable alternative path generation
num_approaches = 7              # Number of paths to explore when stuck
retry_threshold = 3             # Failures before triggering recovery

[conductor]
auto_setup = true               # Initialize project context on first run
context_dir = ".conductor"      # High-level context (product/tech_stack)
tracks_dir = "tracks"           # Per-feature specifications and plans

[mcp]
prefer_osgrep = true            # Use semantic code search
use_sequential_thinking = true  # Enable multi-step reasoning protocol
required_tools = ["chrome-devtools"]

[security]
enforce_allowlist = true        # Use .autocode/security-allowlist.json
allowlist_file = ".autocode/security-allowlist.json"
blocked_patterns = ["rm -rf /", "sudo"] # Absolute constraints

[development]
default_port = 8000             # Default port for dev servers
port_range_start = 8000         # Fallback port range start
port_range_end = 8099           # Fallback port range end
check_module_imports = true     # Verify import/export consistency (JS/TS)
check_console_errors = true     # Check browser console for errors
check_port_availability = true  # Check ports before starting servers

[notifications]
webhook_enabled = true
webhook_url = "https://discord.com/api/webhooks/..."
```

## How It Works: The 5-Phase Loop

When you run `vibe`, the engine determines the next action using a phased approach:

1.  **Phase 1: Init** (`auto-init`) → Populates the database (`.autocode/progress.db`) and basic structure.
2.  **Phase 2: Context** (`auto-context`) → Establishes project-wide product and technical requirements.
3.  **Phase 3: Continue** (`auto-continue`) → Executes the next task in the active `plan.md` (Track mode).
4.  **Phase 4: Completion** → Checks if all features pass; if so, terminates the loop gracefully.
5.  **Phase 5: Transition** (`auto-continue`) → If no active track exists, picks the next failing feature to implement.

## Template Architecture: Progressive Discovery

Command templates use a modular "Progressive Discovery" system to minimize token usage:

```
templates/
├── index.json           # Routing table for project types
├── core/                # Always-included fundamentals
│   ├── identity.md      # Agent identity (6 lines)
│   ├── security.md      # Security constraints (16 lines)
│   └── ...
├── modules/             # Read on-demand by the agent
│   ├── javascript.md    # Web/JS specifics (ports, imports)
│   ├── rust.md          # CLI/Rust patterns
│   ├── testing.md       # Playwright, E2E protocols
│   └── recovery.md      # Stuck protocol
├── commands/
│   ├── auto-init-v2.md     # Lean entry point (~100 lines)
│   └── auto-continue-v2.md # Lean entry point (~80 lines)
```

The agent reads specialized modules only when needed, reducing context window consumption by ~80%.

## Requirements

- **[OpenCode CLI](https://github.com/sst/opencode)**: must be installed and authenticated.
- **Rust Toolchain**: 1.75+ required for building from source.
- **SQLite**: Runtime dependency (usually bundled).

## Installation

```bash
git clone https://github.com/NocturnLabs/opencode-autocode.git
cd opencode-autocode
cargo install --path .
```

---

_Created by NocturnLabs_
