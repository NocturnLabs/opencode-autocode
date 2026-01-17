# Task Backlog

## OpenSpec Proposals to Create

### Proposal 1: `deprecate-legacy-project-config`
- [ ] Create `openspec/changes/deprecate-legacy-project-config/proposal.md`
- [ ] Create `openspec/changes/deprecate-legacy-project-config/tasks.md`
- [ ] Create spec deltas for affected capabilities
- [ ] Run `openspec validate deprecate-legacy-project-config --strict`

### Proposal 2: `orchestrate-reasoning-and-coding-phases`
- [ ] Create `openspec/changes/orchestrate-reasoning-and-coding-phases/proposal.md`
- [ ] Create `openspec/changes/orchestrate-reasoning-and-coding-phases/tasks.md`
- [ ] Define "implementation packet" JSON schema
- [ ] Create spec deltas for autonomous loop capability
- [ ] Run `openspec validate orchestrate-reasoning-and-coding-phases --strict`

### Proposal 3: `improve-autonomous-reliability-and-fallbacks`
- [ ] Create `openspec/changes/improve-autonomous-reliability-and-fallbacks/proposal.md`
- [ ] Create `openspec/changes/improve-autonomous-reliability-and-fallbacks/tasks.md`
- [ ] Define failure classification enum
- [ ] Define fallback chain config schema
- [ ] Create spec deltas
- [ ] Run `openspec validate improve-autonomous-reliability-and-fallbacks --strict`

### Proposal 4: `expand-regression-coverage-for-autonomous-loop`
- [ ] Create `openspec/changes/expand-regression-coverage-for-autonomous-loop/proposal.md`
- [ ] Create `openspec/changes/expand-regression-coverage-for-autonomous-loop/tasks.md`
- [ ] Define test cases for config precedence
- [ ] Define test cases for model selection
- [ ] Define test cases for fallback behavior
- [ ] Run `openspec validate expand-regression-coverage-for-autonomous-loop --strict`

---

## Implementation Tasks (post-approval)

### Config Hygiene
- [ ] Remove `.forger/config.toml` from scaffolding templates
- [ ] Update `opencode.json` template to exclude `.forger/config.toml`
- [ ] Update config loader to only use `forger.toml`
- [ ] Add migration warning for projects with legacy config
- [ ] Remove `LEGACY_CONFIG_FILENAME` constant usage

### Two-Phase Orchestration
- [x] Define `ImplementationPacket` struct (JSON-serializable)
- [x] Create reasoning phase prompt template
- [x] Create coding phase prompt template
- [x] Update `SessionOptions` to support phase context
- [x] Modify supervisor loop to run two phases per feature
- [x] Remove `@coder` references from templates and logging
- [x] Update `dual_model_enabled` semantics or deprecate flag

### Reliability + Fallbacks
- [ ] Add `FailureType` enum (IdleTimeout, SessionTimeout, ExitFailure, VerificationFailed, Unknown)
- [ ] Add `models.reasoning_fallback` and `models.autonomous_fallback` config fields
- [ ] Implement fallback model switching in `handle_session_result`
- [ ] Add `idle_timeout_seconds = 0` support (disable)
- [ ] Add per-phase timeout configuration
- [ ] Add "escalate timeouts after first timeout" option

### Testing
- [ ] Add unit tests for config precedence
- [ ] Add unit tests for effective model selection
- [ ] Add unit tests for `ImplementationPacket` parsing
- [ ] Add regression test: `opencode.json` contains only `forger.toml`
- [ ] Add regression test: `.forger/config.toml` not scaffolded
- [ ] Add regression test: two-phase orchestration flow
- [ ] Run `cargo fmt && cargo clippy`
- [ ] Run full regression suite

---

## Investigation Tasks (if needed)
- [ ] Check OpenCode CLI for any `--coder-model` or similar flags
- [ ] Document OpenCode's actual handling of multiple instruction files
- [ ] Profile token usage before/after two-phase split
