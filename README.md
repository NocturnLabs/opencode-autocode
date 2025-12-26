# OpenCode Autocode

A Rust CLI that scaffolds autonomous coding projects for [OpenCode](https://github.com/sst/opencode) and runs them to completion. It bridges the gap between high-level application specs and fully implemented features.

> [!WARNING]
> **AI-Generated Code Disclaimer**: Significant portions of this codebase (including logic, templates, and tests) were generated or refined using Large Language Models. Use with appropriate caution and always review changes in your local projects.

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
- 🛠️ **MCP Native**: First-class support for Model Context Protocol tools like `osgrep`, `chrome-devtools`, and `sqlite-mcp`.

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
default = "opencode/big-pickle"     # Used for spec generation
autonomous = "opencode/grok-code"  # Used for heart of the coding loop
reasoning = "opencode/grok-code"   # Used for planning and complex decisions
enhancement = "opencode/big-pickle" # Used for discovering improvements

[autonomous]
database_file = ".autocode/progress.db" # Path to progress database
delay_between_sessions = 5      # Seconds to wait between sessions
max_iterations = 0              # 0 = Run until all features pass
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
required_tools = ["chrome-devtools", "sqlite-mcp"]

[security]
enforce_allowlist = true        # Use scripts/security-allowlist.json
allowlist_file = "scripts/security-allowlist.json"
blocked_patterns = ["rm -rf /", "sudo"] # Absolute constraints

[notifications]
webhook_enabled = true
webhook_url = "https://discord.com/api/webhooks/..."
```

## How It Works: The 5-Phase Loop

When you run `vibe`, the engine determines the next action using a phased approach:

1.  **Phase 1: Init** → Runs `auto-init` command to populate the `.autocode/progress.db` and basic structure.
2.  **Phase 2: Context** → (If Conductor enabled) Runs `auto-context` to define product goals and tech stack.
3.  **Phase 3: Work** → Runs `auto-continue` to implement the next task in the active `plan.md`.
4.  **Phase 4: Verify** → Checks database for progress and marks features passing based on session results.
5.  **Phase 5: Plan** → (If no active track) Runs `auto-plan` to create a new track/plan for the next failing feature.

## Requirements

- [OpenCode CLI](https://github.com/sst/opencode) installed and in your PATH.
- Rust toolchain (for building from source).

## Installation

```bash
git clone https://github.com/NocturnLabs/opencode-autocode.git
cd opencode-autocode
cargo install --path .
```

---

_Created by NocturnLabs_
