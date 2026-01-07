# COMMAND: auto-init

{{INCLUDE core/identity.md}}
{{INCLUDE core/security.md}}

---

> [!IMPORTANT] > **DO NOT modify the configuration file (`forger.toml` or `.forger/config.toml`).**
> Your task is to set up the project structure and database, NOT to reconfigure the agent models or settings.

> [!CAUTION]
> **DO NOT create `feature_list.json`.** All features MUST be stored in the SQLite database (`.forger/progress.db`).
> Use `opencode-forger db exec "INSERT INTO features ..."` to add features. File-based tracking is deprecated.

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

**CRITICAL: Break down the specification into SEPARATE, testable features.** Based on your analysis of `app_spec.md` (which defines approximately **{{SPEC_FEATURE_COUNT}}** features and **{{SPEC_ENDPOINT_COUNT}}** API endpoints), insert ALL required features into the database.

> [!TIP]
> **Use batch INSERT to minimize tool calls. Insert 10-50 features per command.**

```bash
opencode-forger db exec "INSERT INTO features (category, description, passes, verification_command) VALUES
  ('functional', 'Feature 1 description', 0, 'bun test -- --grep \"feature1\"'),
  ('functional', 'Feature 2 description', 0, 'bun test -- --grep \"feature2\"'),
  ('functional', 'Feature 3 description', 0, 'bun test -- --grep \"feature3\"')"
```

#### Example: Game Project with 9 Core Features (SINGLE BATCH)

```bash
# ✅ DO: Batch insert all features in ONE command
opencode-forger db exec "INSERT INTO features (category, description, passes, verification_command) VALUES
  ('functional', 'Hero entity spawns and renders as red square', 0, 'cargo test test_hero_spawn'),
  ('functional', 'Hero moves upward automatically at constant speed', 0, 'cargo test test_hero_movement'),
  ('functional', 'Weapon system fires projectiles automatically', 0, 'cargo test test_weapon_firing'),
  ('functional', 'Zombie enemies spawn and move toward hero', 0, 'cargo test test_zombie_spawn'),
  ('functional', 'Collision detection between projectiles and zombies', 0, 'cargo test test_collision'),
  ('functional', 'Gate entities modify weapon properties on contact', 0, 'cargo test test_gate_effects'),
  ('functional', 'SQLite database persists high scores', 0, 'cargo test test_score_persistence'),
  ('style', 'UI displays current score and weapon stats', 0, 'cargo test test_ui_display'),
  ('style', 'Audio plays on weapon fire and gate contact', 0, 'cargo test test_audio')"

# ❌ DON'T: Insert one by one (floods tool calls)
# opencode-forger db exec "INSERT ... VALUES ('functional', 'Feature 1', ...)"
# opencode-forger db exec "INSERT ... VALUES ('functional', 'Feature 2', ...)"
```

#### Requirements

| Rule             | Description                                                            |
| ---------------- | ---------------------------------------------------------------------- |
| **Granularity**  | Each feature = ONE testable behavior. Not "implement the app".         |
| **Count**        | **MINIMUM 200 features** for comprehensive coverage. Cover every requirement exhaustively. |
| **Depth**        | At least **25 features MUST have 10+ testing steps** for thorough validation. |
| **Categories**   | Mix `functional` (logic) and `style` (UI/UX)                           |
| **Passes**       | ALL start with `passes = 0`                                            |
| **Verification** | Project-appropriate commands (e.g. `npm test`, `pytest`, `cargo test`) |

> [!IMPORTANT]
> **IT IS CATASTROPHIC TO REMOVE OR EDIT FEATURES IN FUTURE SESSIONS.**
> Features can ONLY be marked as passing. Never remove, never edit descriptions, never modify testing steps.

> [!TIP] > **Use standard test runners:**
>
> - **Node/TS:** `npm test -- --grep "feature name"` or `vitest run -t "feature"`
> - **Python:** `pytest tests/e2e/test_name.py`
> - **Rust:** `cargo test --test feature_name`
> - **Custom:** Create a `verify.sh` that takes a feature name/ID.

> **NEVER** combine multiple behaviors into one feature.
> **NEVER** use generic verification like `cargo build` — use actual test commands.
> **MONOREPO/NESTED PATHS:** If you create subdirectories (e.g., `backend/`, `frontend/`), your verification commands MUST reference them.
> - **Wrong:** `cargo test` (fails if Cargo.toml is in backend/)
> - **Right:** `cd backend && cargo test` OR `npm test --prefix frontend`
> **Ensure verification commands are executable from the root directory.**
> **For web projects:** E2E tests (Playwright) are MANDATORY. See `templates/modules/testing.md`.
> **Need examples?** Run `opencode-forger example db --insert` or `example db --query`.
> **Stuck already?** Run `opencode-forger example vibe` for orientation.

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
> If your `init.sh` uses a simple file server (like `python3 -m http.server`), ensure `index.html` is in the project root or explicitly specify the directory (e.g., `-d public`).
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
