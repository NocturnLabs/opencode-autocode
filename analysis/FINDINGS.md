# Validated Findings

## 1. Model Selection Confusion

**Issue**: Users configure `[models].autonomous` expecting it to be used for `opencode run --model ...`, but when dual-model is enabled (default), the code uses `config.models.reasoning` instead.

**Evidence** (from `src/autonomous/settings.rs:176-180`):
```rust
settings.model = if settings.dual_model_enabled {
    config.models.reasoning.clone()
} else {
    config.models.autonomous.clone()
};
```

**Log evidence** (from `forger-gh-wrapped/opencode-debug-840977.log`):
```
INFO  Model: opencode/minimax-m2.1-free
INFO  Dual-model: enabled (reasoning + @coder)
```
But `forger.toml` specifies:
```toml
autonomous = "xiaomi/mimo-v2-flash"
reasoning = "xiaomi/mimo-v2-flash"
```
The supervisor *actually* used the reasoning model field, not autonomous.

**Root cause**: The naming and semantics are unclear. "Dual-model" implied `@coder` subagent usage, but `@coder` is being deprecated.

---

## 2. Conflicting Config Instructions

**Issue**: Generated `opencode.json` passes BOTH `forger.toml` AND `.forger/config.toml` as instructions to OpenCode.

**Evidence** (from `forger-reprompt/opencode.json`):
```json
"instructions": [
  "forger.toml",
  ".forger/config.toml",
  "/home/yum/.../app_spec.md"
]
```

**Problem**: These files can have different values for the same settings. The Rust config loader prefers `forger.toml`, but OpenCode's agent receives both as context, leading to contradictory guidance.

**Resolution**: Deprecate `.forger/config.toml` entirely. Only use `forger.toml`.

---

## 3. Timeout-Related Failures

**Issue**: Many autonomous runs are killed by idle or session timeouts during legitimate work (builds, tests, tool invocations).

**Evidence** (from logs):
```
DEBUG Session timeout: 15m, Idle timeout: 600s
...
INFO  Idle timeout after 600 seconds of silence
```

**Defaults** (from `src/config/autonomous.rs:55-56`):
```rust
session_timeout_minutes: 15,
idle_timeout_seconds: 600,
```

**Problem**: 600s of silence is common during:
- `npm install` / dependency resolution
- Long test suites
- Build compilation
- Tool initialization (MCP servers)

**Resolution**: Make timeouts phase-aware and/or disable idle timeout by default for certain phases.

---

## 4. Retry Without Model Escalation

**Issue**: The autonomous loop retries with exponential backoff but does not switch models on repeated failures.

**Evidence** (from `src/autonomous/settings.rs:140-143`):
```rust
let exponent = (*consecutive_errors - 1).min(6);
let backoff = settings.delay_seconds.saturating_mul(1 << exponent);
println!("-> Retrying in {}s (exponential backoff)...", backoff);
LoopAction::RetryWithBackoff(backoff)
```

No model change occurs. Compare to spec generation (`src/services/generator/executor.rs:64-68`) which does switch to `config.models.fixer` on retry.

**Resolution**: Add configurable fallback model chains for autonomous sessions.

---

## 5. @coder Subagent Deprecation

**Issue**: The workflow references `@coder` as a subagent, but this is being deprecated. The intended workflow is:
1. **Reasoning model**: Produces implementation plan/instructions
2. **Coding model**: Executes tool calls to implement the plan

**Evidence**: User feedback confirmed this is the desired architecture.

**Resolution**: Remove `@coder` references, implement explicit two-phase orchestration.

---

## Code Pointers

| Area | File | Lines |
|------|------|-------|
| Model selection | `src/autonomous/settings.rs` | 176-180 |
| Timeout defaults | `src/config/autonomous.rs` | 55-56 |
| Retry logic | `src/autonomous/settings.rs` | 126-151 |
| Fixer model retry | `src/services/generator/executor.rs` | 64-68 |
| Session execution | `src/autonomous/session.rs` | 44-80 |
| Supervisor loop | `src/autonomous/supervisor/loop.rs` | (main loop) |
| Config loading | `src/config/mod.rs` | 102-133 |
| Config precedence | `src/config/mod.rs` | 260-277 |
