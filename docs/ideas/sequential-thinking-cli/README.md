# Sequential Thinking CLI Replacement

> **Status**: Idea / Not Yet Implemented  
> **Created**: 2025-12-26  
> **Goal**: Replace `sequential-thinking` MCP with native CLI commands to reduce context bloat.

---

## Problem Statement

MCP tool calls add significant context overhead (~200-500 tokens per call for schema + response wrapper). The `sequential-thinking` MCP is used for structured problem-solving, but its benefits can be replicated with CLI commands that:
1. Persist thinking across sessions (SQLite)
2. Allow prior analysis lookup before re-analyzing
3. Self-document via `--help`

---

## Research: What Sequential Thinking MCP Does

The sequential-thinking MCP provides:

| Feature | Description |
| :--- | :--- |
| **Structured stages** | Problem Definition → Research → Analysis → Synthesis → Conclusion |
| **Thought tracking** | Records each step with sequence number, stage, progress |
| **Revision/branching** | Allows revisiting earlier steps, exploring alternatives |
| **Context persistence** | Feeds history back to avoid circular reasoning |
| **Summary generation** | Concise overviews of thought process |

**Sources**: LobeHub, MCPServers.org, Glama.ai, Medium

---

## Proposed Design

### CLI Interface

```bash
# Start a new thinking session
opencode-autocode think new "Problem description"

# Add a thought with stage
opencode-autocode think add <session_id> --stage analysis "Thought content"

# List sessions or thoughts
opencode-autocode think list
opencode-autocode think list <session_id>

# Branch from existing thought
opencode-autocode think branch <thought_id> "Alternative approach"

# Get summary for context injection
opencode-autocode think summarize <session_id>

# Mark complete
opencode-autocode think complete <session_id>

# Search prior analysis
opencode-autocode think search "query"
```

### Database Schema

```sql
CREATE TABLE IF NOT EXISTS thinking_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    problem TEXT NOT NULL,
    status TEXT DEFAULT 'active',  -- active, complete
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    parent_id INTEGER,  -- NULL for root, set for branches
    stage TEXT NOT NULL,  -- problem, research, analysis, synthesis, conclusion
    content TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES thinking_sessions(id),
    FOREIGN KEY (parent_id) REFERENCES thoughts(id)
);
```

### New Files

| File | Purpose |
| :--- | :--- |
| `src/thinking/mod.rs` | `ThinkingSession`, `Thought`, `ThinkingRepository` |
| `src/db/thinking.rs` | Database operations for thinking tables |
| `templates/modules/thinking.md` | Progressive discovery guide for agent |

---

## Pros and Cons

### ✅ Pros

- **Context savings**: CLI output is raw text, no MCP schema overhead
- **Persistence**: SQLite survives across sessions (MCP is ephemeral)
- **Searchable**: Agent can check prior analysis before re-analyzing
- **No external dependency**: Fewer moving parts
- **Auditable**: Full thinking history for debugging

### ❌ Cons

- **Loses active guidance**: MCP prompts through stages; CLI is passive
- **Maintenance burden**: You own the thinking logic
- **Integration friction**: CLI calls less "native" to agent tool flow

### ⚠️ Footguns

| Risk | Mitigation |
| :--- | :--- |
| Agent forgets to use | Template: "Run `think search` before complex work" |
| Stale thinking | Add `updated_at`; warn if session > 7 days old |
| Lost scaffolding | CLI enforces stage enum validation |
| Search false positives | Use SQLite FTS5; limit to same project |

---

## Decision Rationale

**Proceed because:**
1. LLMs already have strong reasoning—MCP guidance is training wheels
2. `--help` is self-documenting, cheaper than MCP schemas in context
3. Worst case: agent under-utilizes it, same as not having it
4. Templates can enforce "check `think search` before complex work"

---

## Implementation Checklist

When ready to implement:

- [ ] Add schema to `src/db/schema.rs`
- [ ] Create `src/thinking/mod.rs` with repository
- [ ] Add `Think` subcommand to `src/cli.rs`
- [ ] Route in `src/main.rs`
- [ ] Create `templates/modules/thinking.md`
- [ ] Write integration tests
- [ ] Update README
