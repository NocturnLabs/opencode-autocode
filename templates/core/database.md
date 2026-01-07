# Database Operations

Features are tracked in `.forger/progress.db`. Use the CLI:

> [!IMPORTANT]
> - **`db query`** = SELECT (read data)
> - **`db exec`** = INSERT, UPDATE, DELETE (write data)
> 
> Using `db exec` for SELECT will return "0 row(s) affected" instead of data!

```bash
# ✅ READ features (use db query)
opencode-forger db query "SELECT id, description FROM features WHERE passes = 0 ORDER BY id LIMIT 1"
opencode-forger db query "SELECT COUNT(*) FROM features WHERE passes = 0"

# ✅ WRITE to database (use db exec)
opencode-forger db exec "UPDATE features SET verification_command = 'new cmd' WHERE id = 1"

# ❌ WRONG - this returns "0 rows affected" instead of data
# opencode-forger db exec "SELECT * FROM features"
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
opencode-forger db knowledge set APP_PORT 3000 --category "environment" --description "Development server port"

# Retrieve a fact
opencode-forger db knowledge get APP_PORT

# List all facts
opencode-forger db knowledge list

# List by category
opencode-forger db knowledge list --category "environment"

# Delete a fact
opencode-forger db knowledge delete APP_PORT
```

### Knowledge Workflow

1. **At session start**: Run `opencode-forger db knowledge list` to recall saved facts.
2. **During implementation**: When you discover or configure something important, SAVE IT.
3. **Before session end**: Review what you learned and save any new facts.
