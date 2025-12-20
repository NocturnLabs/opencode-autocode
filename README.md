# OpenCode Autocode 🎵

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

## Commands

```bash
# Scaffolding
opencode-autocode --interactive       # TUI to build spec
opencode-autocode --default           # Use default template
opencode-autocode --spec FILE         # Use custom spec

# Running
opencode-autocode vibe                # Start autonomous loop
opencode-autocode vibe --limit 10     # Limit iterations

# Utilities
opencode-autocode --config            # Configure via TUI
opencode-autocode --regression-check  # Verify features
opencode-autocode templates list      # List templates
```

## Configuration

Edit `autocode.toml`:

```toml
[models]
autonomous = "anthropic/claude-sonnet-4"

[autonomous]
delay_between_sessions = 5
max_iterations = 0  # 0 = unlimited

[generation]
complexity = "comprehensive"  # or "minimal"
min_features = 15
```

## How It Works

1. **Scaffold** → Creates `app_spec.md`, `.opencode/commands/`, `autocode.toml`
2. **Vibe** → Runs loop:
   - First run: `opencode run --command auto-init` (creates `feature_list.json`)
   - Subsequent: `opencode run --command auto-continue` (implements features)
   - All passing: Exit

Each session picks one failing feature, implements it, verifies, marks passing, commits. If stuck after 3 retries, generates alternative approaches.

## Installation

```bash
git clone https://github.com/nocturnlabs/opencode-autocode.git
cd opencode-autocode
cargo install --path .
```

---

_NocturnLabs_
