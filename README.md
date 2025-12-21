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

- 🚀 **Zero-Config Scaffolding**: TUI to build app specs in seconds.
- 🔄 **Vibe Loop**: Autonomous session management with automatic continuation.
- 🔁 **Stuck Recovery**: Generates alternative approaches when the agent gets stuck.
- 🔔 **Optional Webhooks**: Get notified when features complete with beautiful Discord embeds.

## Configuration

Edit `autocode.toml` or use `opencode-autocode --config`.

```toml
[models]
autonomous = "opencode/grok-code"

[autonomous]
delay_between_sessions = 5
max_iterations = 0  # 0 = unlimited

[notifications]
webhook_enabled = true
webhook_url = "https://discord.com/api/webhooks/..."
```

## How It Works

1. **Scaffold** → Creates `app_spec.md`, `.opencode/commands/`, `autocode.toml`
2. **Vibe** → Runs loop:
   - First run: `opencode run --command auto-init` (creates `feature_list.json`)
   - Subsequent: `opencode run --command auto-continue` (implements features)
   - **Notify**: Detects newly passing features and fires a webhook notification.
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
