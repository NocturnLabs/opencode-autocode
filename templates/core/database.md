# Database Operations

Features are tracked in `.autocode/progress.db`. Use the CLI:

```bash
# Query features
opencode-autocode db query "SELECT id, description FROM features WHERE passes = 0 ORDER BY id LIMIT 1"

# Count remaining
opencode-autocode db query "SELECT COUNT(*) FROM features WHERE passes = 0"

# Mark feature as passing (after verification)
opencode-autocode db mark-pass X
```

**YOU CAN ONLY CHANGE THE `passes` FIELD. NEVER delete or edit feature descriptions.**

---

## Knowledge Base (Persistent Memory)

The `knowledge` table stores facts that persist across sessions. **USE THIS ACTIVELY.**

### When to Save Knowledge

> [!IMPORTANT]
> Save knowledge whenever you discover environment-specific information that future sessions will need.

**ALWAYS save:**

- Port numbers used by the application (e.g., `APP_PORT=3000`)
- Database connection details (e.g., `DB_PATH=./data/app.db`)
- API keys or secret locations (e.g., `API_KEY_ENV=OPENAI_API_KEY`)
- Build commands that differ from defaults (e.g., `BUILD_CMD=pnpm build`)
- Test commands (e.g., `TEST_CMD=bun x playwright test`)
- Any workarounds or fixes discovered (e.g., `QUIRK_VITE_PORT=Must use --port flag`)

### Knowledge Commands

```bash
# Save a fact
opencode-autocode db knowledge set APP_PORT 3000 --category "environment" --description "Development server port"

# Retrieve a fact
opencode-autocode db knowledge get APP_PORT

# List all facts
opencode-autocode db knowledge list

# List by category
opencode-autocode db knowledge list --category "environment"

# Delete a fact
opencode-autocode db knowledge delete APP_PORT
```

### Knowledge Workflow

1. **At session start**: Run `opencode-autocode db knowledge list` to recall saved facts.
2. **During implementation**: When you discover or configure something important, SAVE IT.
3. **Before session end**: Review what you learned and save any new facts.
