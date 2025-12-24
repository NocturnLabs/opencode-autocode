# OpenCode Autocode

A Rust CLI that scaffolds autonomous coding projects for [OpenCode](https://github.com/sst/opencode) and runs them to completion.

## Quick Start

```bash
# 1. Create project
opencode-autocode --interactive

# 2. Configure (optional)
opencode-autocode --config

# 3. Vibe
opencode-autocode vibe
```

That's it. The tool generates a spec, creates project structure, then autonomously implements features one by one until done.

### Running

- `opencode-autocode vibe` → Start autonomous loop
- `opencode-autocode vibe --limit 10` → Limit iterations
- `opencode-autocode --regression-check` → Verify passing features

### Features

- 🚀 **Zero-Config Scaffolding**: TUI to build app specs in seconds
- 🔄 **Vibe Loop**: Autonomous session management with automatic continuation
- ⏱️ **Session Timeout**: Kill hung sessions after configurable timeout
- ✅ **Auto-Commit**: Automatically commit completed features to git
- 🔁 **Stuck Recovery**: Generates alternative approaches when stuck
- 🔔 **Webhooks**: Get notified when features complete (Discord/Slack)
- 🛠️ **MCP Integration**: Configure OpenCode MCP servers via `opencode.json`

## Configuration

Run `opencode-autocode --config` for interactive setup, or edit `autocode.toml` directly.

```toml
[models]
autonomous = "opencode/grok-code"

[autonomous]
delay_between_sessions = 5
max_iterations = 0              # 0 = unlimited
session_timeout_minutes = 60    # 0 = no timeout
auto_commit = true              # Commit on feature completion

[mcp]
prefer_osgrep = true            # Use semantic code search
use_sequential_thinking = true  # Complex reasoning MCP
required_tools = ["chrome-devtools"]  # For web projects

[notifications]
webhook_enabled = true
webhook_url = "https://discord.com/api/webhooks/..."
```

The config TUI also generates `opencode.json` with MCP server settings based on your preferences.

## How It Works

1. **Scaffold** → Creates `app_spec.md`, `.opencode/commands/`, `autocode.toml`, `opencode.json`
2. **Vibe** → Runs loop:
   - First run: `opencode run --command auto-init` (creates `feature_list.json`)
   - Subsequent: `opencode run --command auto-continue` (implements features)
   - **Auto-commit**: Commits completed features to git
   - **Notify**: Fires webhook on feature completion
   - All passing: Exit

Each session picks one failing feature, implements it, verifies, marks passing, commits. If stuck after 3 retries, generates alternative approaches.

## Installation

```bash
git clone https://github.com/NocturnLabs/opencode-autocode.git
cd opencode-autocode
cargo install --path .
```

---

_NocturnLabs_
