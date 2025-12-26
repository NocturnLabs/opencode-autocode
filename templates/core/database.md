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
