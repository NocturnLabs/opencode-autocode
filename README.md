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
- `opencode-autocode vibe --developer` → Enable comprehensive debug logging
- `opencode-autocode vibe --limit 10` → Limit iterations
- `opencode-autocode --regression-check` → Verify passing features

### Features

- 🚀 **Zero-Config Scaffolding**: TUI to build app specs in seconds
- 🔄 **Vibe Loop**: Autonomous session management with automatic continuation
- 🧠 **Conductor Workflow**: Context-driven planning with persistent `plan.md` artifacts
- 📝 **Developer Mode**: Comprehensive logging of subprocess output to `opencode-debug.log`
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

[conductor]
auto_setup = true               # Create .conductor/ context and tracks/ automatically
context_dir = ".conductor"      # Persistent project context
tracks_dir = "tracks"           # Per-feature specs and plans

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
2. **Vibe** → Runs loop with 5-phase command determination:
   - **Phase 1: Init** → `auto-init` (creates `feature_list.json`)
   - **Phase 2: Context** → `auto-context` (creates `.conductor/` product/tech/workflow docs)
   - **Phase 3: Work** → `auto-continue` (implements features using `tracks/plan.md`)
   - **Phase 4: Verify** → Check `feature_list.json` status
   - **Phase 5: Plan** → `auto-plan` (creates new track/plan for failing features)
3. **Finish** → All passing: Exit

Each session picks one failing feature, implements it, verifies, marks passing, commits. If stuck, generates alternative approaches. Use `--developer` to see everything happening under the hood.

## Installation

```bash
git clone https://github.com/NocturnLabs/opencode-autocode.git
cd opencode-autocode
cargo install --path .
```

---

_NocturnLabs_
