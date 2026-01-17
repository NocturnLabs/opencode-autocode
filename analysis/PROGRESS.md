# Progress Tracker

**Last updated**: 2026-01-16

## Current Phase
Proposal 1 (deprecate-legacy-project-config) - **COMPLETED**; validating Proposal 2.

## Completed
- [x] Log analysis across `forger-reprompt` and `forger-gh-wrapped` projects
- [x] Identified root causes (model selection, config conflicts, timeouts, retry gaps)
- [x] Reviewed relevant source code (`config/`, `autonomous/`, `services/generator/`)
- [x] Drafted and submitted implementation plan
- [x] Plan approved with feedback incorporated
- [x] Created `analysis/` workspace (this directory)
- [x] **Proposal 1**: `deprecate-legacy-project-config` - COMPLETED
  - Removed `LEGACY_CONFIG_FILENAME` constant
  - Updated `Config::load()` to only use `forger.toml`
  - Added deprecation warning when `.forger/config.toml` exists
  - Updated scaffolder to stop creating `.forger/config.toml`
  - Updated `generate_opencode_json()` to exclude legacy config
  - Updated config editor to only save to `forger.toml`

## In Progress
- [ ] **Proposal 2**: `orchestrate-reasoning-and-coding-phases` (reviewing)

## Next Up
- [ ] Implement Proposal 2: Two-phase orchestration (reasoning → coding)
- [ ] Implement Proposal 3: Reliability and fallbacks
- [ ] Implement Proposal 4: Regression tests
- [ ] Run `cargo fmt && cargo clippy` (after all proposals)

## Blockers
None currently.

## Key Decisions Made
1. **JSON** chosen for "implementation packet" format (user approved).
2. `.forger/config.toml` will be **fully deprecated** (not compatibility-mirrored).
3. `@coder` subagent concept removed; replaced with explicit two-phase Reasoning → Coding workflow.
4. `analysis/` is local-only debugging workspace, NOT product functionality.
5. **Proposal 1 implementation completed** - core deprecation functionality done.

## Files Modified This Session
**Proposal 1 (deprecate-legacy-project-config) - COMPLETED:**
- `analysis/PROGRESS.md` (updated)
- `src/config/mod.rs`:
  - Removed `LEGACY_CONFIG_FILENAME` constant
  - Updated `Config::load()` - now only reads `forger.toml`, warns about `.forger/config.toml`
  - Updated `resolve_config_path()` - simplified to only check `forger.toml`
- `src/services/scaffold/mod.rs`:
  - Removed `.forger/config.toml` creation in `scaffold_with_spec_text()`
  - Updated `generate_opencode_json()` - removed `.forger/config.toml` from instructions
- `src/config_tui/mod.rs`:
  - Removed dual-save logic for `.forger/config.toml`
  - Updated help text to not mention `.forger/config.toml`

## Tests Added
- `src/config/mod.rs`:
  - `test_config_loader_ignores_legacy_config()` - Verifies forger.toml is used when legacy exists
  - `test_config_loader_with_only_legacy_config()` - Verifies default is used when only legacy exists

## Context for Next Session
Proposal 1 is **COMPLETED**. Core deprecation functionality implemented:
- Legacy config (`.forger/config.toml`) is no longer generated or read
- Single source of truth: `forger.toml` at project root
- Migration warning shown when `.forger/config.toml` exists
- Config TUI only writes to `forger.toml`
- Generated `opencode.json` only includes `forger.toml` in instructions
- Tests verify config precedence behavior

Next: Review Proposal 2 to begin implementation of two-phase orchestration.
