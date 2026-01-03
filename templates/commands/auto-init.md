# COMMAND: auto-init

{{INCLUDE core/identity.md}}
{{INCLUDE core/security.md}}

---

> [!IMPORTANT] > **DO NOT modify the configuration file (`autocode.toml` or `.autocode/config.toml`).**
> Your task is to set up the project structure and database, NOT to reconfigure the agent models or settings.

This is the FIRST session. Set up the foundation for all future sessions.

### STEP 1: Read Project Specification

Read `app_spec.md` in your working directory. Understand:

- What the project does
- Technology stack
- Features to build
- Success criteria

**Detect project type** to know which modules you'll need later.

---

### STEP 2: Populate Features Database

**CRITICAL: Break down spec into 5-15 SEPARATE features.** Each feature = one testable unit.

Based on `app_spec.md`, insert features using the CLI:

```bash
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Feature description', 0, 'bun test -- --grep \"feature\"')"
```

#### Example: Game Project with 9 Core Features

```bash
# DON'T: One vague feature
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Implement the game', 0, 'cargo build')"

# DO: Separate testable features
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Hero entity spawns and renders as red square', 0, 'cargo test test_hero_spawn')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Hero moves upward automatically at constant speed', 0, 'cargo test test_hero_movement')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Weapon system fires projectiles automatically', 0, 'cargo test test_weapon_firing')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Zombie enemies spawn and move toward hero', 0, 'cargo test test_zombie_spawn')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Collision detection between projectiles and zombies', 0, 'cargo test test_collision')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'Gate entities modify weapon properties on contact', 0, 'cargo test test_gate_effects')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('functional', 'SQLite database persists high scores', 0, 'cargo test test_score_persistence')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('style', 'UI displays current score and weapon stats', 0, 'cargo test test_ui_display')"
opencode-autocode db exec "INSERT INTO features ... VALUES ('style', 'Audio plays on weapon fire and gate contact', 0, 'cargo test test_audio')"
```

#### Requirements

| Rule             | Description                                                    |
| ---------------- | -------------------------------------------------------------- |
| **Granularity**  | Each feature = ONE testable behavior. Not "implement the app". |
| **Count**        | Insert 5-15 features minimum. More is better.                  |
| **Categories**   | Mix `functional` (logic) and `style` (UI/UX)                   |
| **Passes**       | ALL start with `passes = 0`                                    |
| **Verification** | Real test commands, not just `cargo build` or `bun run dev`    |

> **NEVER** combine multiple behaviors into one feature.
> **NEVER** use generic verification like `cargo build` — use actual test commands.
> **For web projects:** E2E tests (Playwright) are MANDATORY. See `templates/modules/testing.md`.
> **Need examples?** Run `opencode-autocode example db --insert` or `example db --query`.
> **Stuck already?** Run `opencode-autocode example vibe` for orientation.

---

### STEP 3: Create Conductor Context

Create `.conductor/` directory with context files:

```bash
mkdir -p .conductor tracks
```

Create `product.md`, `tech_stack.md`, `workflow.md` based on the spec.

---

### STEP 4: Create init.sh (One-Click Install + Launch)

Create a **one-click** setup script that:

1. **Installs** all dependencies (npm install, cargo build, go mod download, etc.)
2. **Launches** the application in development mode

> [!IMPORTANT] > `init.sh` MUST be a single-command way to go from "clone" to "running application".
> Users should be able to run `./init.sh` and see the application working.

**Example structure:**

```bash
#!/bin/bash
set -e

# Install dependencies
npm install # or cargo build, pip install -r requirements.txt, etc.

# Launch application
npm run dev # or cargo run, python app.py, etc.
```

> **Web projects:** Include port conflict prevention. Always prefer finding a free port over killing existing ones.
> See `templates/modules/javascript.md` for template.

---

### STEP 5: Create .gitignore

Standard patterns for your tech stack (node_modules, target, **pycache**, etc.)
Be sure to include `.osgrep/` in the ignore list.

---

### STEP 6: Initialize Git

```bash
git init
git add .
git commit -m "Initial setup: features database, conductor context, project structure"
```

---

### STEP 7: Signal Continuation

```bash
echo "CONTINUE" > .opencode-signal
```

Output: `===SESSION_COMPLETE===`

---

## Help Index

| Situation              | Module                            |
| ---------------------- | --------------------------------- |
| Web/JavaScript project | `templates/modules/javascript.md` |
| Testing setup          | `templates/modules/testing.md`    |
| MCP tool usage         | `templates/core/mcp_guide.md`     |
| Orientation            | `example vibe`                    |
| Tracks                 | `example tracks`                  |
| Templates              | `example templates-guide`         |
