# Analysis Workspace

This directory is a **local-only debugging workspace** for investigating and fixing issues in opencode-forger's autonomous loop. It is **NOT** product functionality and should not be:
- Scaffolded into user projects
- Referenced in product documentation
- Required for builds or tests

## Purpose

When working on complex multi-session investigations (e.g., model selection issues, timeout problems, config conflicts), this folder provides continuity between AI assistant sessions when context windows reset.

## Files

| File | Purpose |
|------|---------|
| `README.md` | This file - how to use the workspace |
| `FINDINGS.md` | Validated findings with evidence (log excerpts, code pointers) |
| `TASKLIST.md` | Full backlog of investigation + implementation tasks |
| `PROGRESS.md` | Current status, what's done, what's next |

## How to Use (for AI assistants)

1. **Starting a new session**: Read `PROGRESS.md` first to understand current state, then `FINDINGS.md` for context.
2. **During work**: Update `PROGRESS.md` as tasks complete. Add new findings to `FINDINGS.md`.
3. **Ending a session**: Ensure `PROGRESS.md` reflects the latest state so the next session can pick up cleanly.

## Related OpenSpec Proposals

Active proposals created from this analysis:
- `openspec/changes/deprecate-legacy-project-config/` - Remove `.forger/config.toml`
- `openspec/changes/orchestrate-reasoning-and-coding-phases/` - Two-phase model workflow
- `openspec/changes/improve-autonomous-reliability-and-fallbacks/` - Retry/fallback strategy
- `openspec/changes/expand-regression-coverage-for-autonomous-loop/` - Test coverage
