# AGENTS.md

This file serves as a guide for OpenCode autonomous agents. It describes the project structure, architectural decisions, and coding patterns to ensure consistency across the codebase.

## Project Structure

- `.forger/`: detailed specs, configuration, and logs for the forger tool.
- `.opencode/`: OpenCode internal configuration and subagent definitions.
- `src/`: Source code directory.
- `tests/`: Test suite.
- `docs/`: Documentation.

## Database Schema

The project uses SQLite for progress tracking. The database is located at `.forger/progress.db`.

### features table
```sql
CREATE TABLE features (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    description TEXT NOT NULL UNIQUE,  -- Feature description (use this, NOT "name")
    passes INTEGER DEFAULT 0,          -- Pass count (0 = failing, >=1 = passing)
    verification_command TEXT,         -- Command to verify the feature
    last_error TEXT,                   -- Last error message if verification failed
    created_at TEXT,
    updated_at TEXT
);
```

> **IMPORTANT**: There is NO `name` column. Use `description` to identify features.

### Useful queries
```sql
-- Get feature by ID
SELECT id, description, verification_command FROM features WHERE id = ?;

-- List all failing features
SELECT id, description, last_error FROM features WHERE passes = 0;

-- Mark feature as passing
UPDATE features SET passes = passes + 1 WHERE id = ?;
```

## Coding Patterns

- **Language**: [Specify Language, e.g., Rust, TypeScript]
- **Style**: Follow standard idioms (e.g., `rustfmt`, `prettier`).
- **Error Handling**: Use distinct error types and propagation.
- **Testing**: thorough unit execution and integration testing.

## Task Workflow

1.  **Plan**: Analyze requirements and update `implementation_plan.md`.
2.  **Edit**: precise code changes using available tools.
3.  **Verify**: Run tests and validate fixes.

## Agent Persona

You are an expert software engineer built by OpenCode. You are precise, proactive, and follow instructions carefully.
