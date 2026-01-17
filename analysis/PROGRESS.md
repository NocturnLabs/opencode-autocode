# Progress Tracker

**Last updated**: 2026-01-17

## Current Phase
Proposal 2 (orchestrate-reasoning-and-coding-phases) - **COMPLETED**

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
- [x] **Proposal 2**: `orchestrate-reasoning-and-coding-phases` - COMPLETED
  - Two-phase orchestration implemented in `src/autonomous/supervisor/two_phase.rs`
  - `ImplementationPacket` JSON schema defined for reasoning → coding handoff
  - Removed deprecated `@coder` subagent references throughout codebase
  - Removed `coder.xml` template and `CODER_AGENT` asset
  - Updated scaffold to no longer generate `coder.md`
  - Cleaned up `dual_model` parameters from template functions
  - `--single-model` CLI flag controls single vs two-phase mode
  - Updated documentation (README.md, ARCHITECTURE.md, INTERNAL_ARCHITECTURE.md)

## In Progress
- [ ] **Proposal 3**: `improve-autonomous-reliability-and-fallbacks`

## Next Up
- [ ] Implement Proposal 3: Reliability and fallbacks
- [ ] Implement Proposal 4: Regression tests

## Blockers
None currently.

## Key Decisions Made
1. **JSON** chosen for "implementation packet" format (user approved).
2. `.forger/config.toml` will be **fully deprecated** (not compatibility-mirrored).
3. `@coder` subagent concept removed; replaced with explicit two-phase Reasoning → Coding workflow.
4. `analysis/` is local-only debugging workspace, NOT product functionality.
5. **Proposal 1 implementation completed** - core deprecation functionality done.
6. **Proposal 2 implementation completed** - two-phase orchestration fully operational.

## Files Modified This Session
**Proposal 2 (orchestrate-reasoning-and-coding-phases) - COMPLETED:**
- `templates/scaffold/agents/coder.xml` - DELETED (deprecated @coder subagent)
- `templates/commands/auto-fix.xml` - Removed `{{dual_model_instructions}}` placeholder
- `src/services/scaffold/assets.rs`:
  - Removed `CODER_AGENT` constant and `coder_agent()` function
- `src/services/scaffold/mod.rs`:
  - Removed `coder.md` generation from scaffolding
- `src/autonomous/templates.rs`:
  - Removed `_dual_model` parameter from `generate_fix_template()`
  - Removed `_dual_model` parameter from `generate_continue_template()`
  - Removed `dual_model_section` and `dual_model_instructions` dead code
- `src/autonomous/supervisor/actions.rs`:
  - Updated function calls to remove dual_model parameter
- `src/autonomous/supervisor/loop.rs`:
  - Updated function calls to remove dual_model parameter
- `README.md`:
  - Updated dual-model architecture description
- `docs/INTERNAL_ARCHITECTURE.md`:
  - Removed `coder.md` from directory structure
  - Updated ADR-003 consequences to reflect two-phase orchestration

## Tests
- Existing tests in `src/autonomous/supervisor/two_phase.rs` verify:
  - `ImplementationPacket` serialization/deserialization
  - Packet validation (feature_id, description, verification_command)
  - JSON parsing from string input

## Context for Next Session
Proposal 2 is **COMPLETED**. Two-phase orchestration fully implemented:
- Reasoning phase produces structured `ImplementationPacket` JSON
- Coding phase executes the packet with the coding model
- `--single-model` flag allows bypassing reasoning phase
- All `@coder` subagent references removed
- Clean separation: `models.reasoning` for planning, `models.autonomous` for execution

Next: Review Proposal 3 to begin implementation of reliability and fallbacks.
