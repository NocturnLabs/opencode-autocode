# Autonomous Work Session

{{INCLUDE core/identity.md}}
{{INCLUDE core/security.md}}

---

## Workflow

### 1. Initialize Environment

**FIRST: Run init.sh if it exists:**

```bash
[ -f ./init.sh ] && chmod +x ./init.sh && ./init.sh
```

Then orient:

- List files, read `app_spec.md`, detect project type
- **Orient**: `opencode-autocode example vibe`
- Query: `opencode-autocode db stats`
- Check `.autocode/session.log` for prior notes

> **Need guidance?** Read `templates/modules/[javascript|rust|testing].md`

---

### 2. Recall & Verify

> [!IMPORTANT] > **MANDATORY: Check your persistent memory before doing ANY work.**

**Step 2a - Recall Knowledge:**

```bash
opencode-autocode db knowledge list
```

Review saved facts (ports, env vars, workarounds). Use these when implementing.

**Step 2b - Regression Check:**

```bash
opencode-autocode db check
```

Verify existing features still pass before new work.

---

### 3. Pick ONE Feature

**CRITICAL: Work on ONE feature only. Do not batch-complete multiple features.**

```sql
SELECT id, description, verification_command FROM features WHERE passes = 0 ORDER BY id LIMIT 1
```

Note the `verification_command` - you MUST run it before marking as passing.

---

### 4. Implement Using @coder

**Delegate implementation to the coding subagent:**

1. Plan the implementation (files, functions, structure)
2. Delegate with detailed spec:

   ```
   @coder Create src/components/Button.tsx that exports a Button component with:
   - Props: label (string), onClick (function), variant ('primary' | 'secondary')
   - Uses Tailwind classes for styling
   - Includes hover and focus states
   ```

3. Review the output
4. If issues, provide specific fix instructions:
   ```
   @coder Fix Button.tsx: add aria-label prop for accessibility
   ```

> **Note:** @coder only implements. All design decisions stay with you.

---

### 5. Verify @coder Output

After @coder completes:

- Review the generated code
- Run tests to verify correctness
- If @coder fails twice, implement directly and note the issue

> **Stuck 3+ times?** Read `templates/modules/recovery.md`

---

### 6. Start Server (Web Projects - MANDATORY)

**Before starting ANY server, check ports:**

```bash
ss -tlnH "sport = :8000" | grep -q . && echo "8000 IN USE" || echo "8000 free"
```

If port in use, find a free one:

```bash
PORT=8000; while ss -tlnH "sport = :$PORT" | grep -q .; do PORT=$((PORT+1)); done; echo "Use port $PORT"
```

Start server on the free port, not the default.

> [!WARNING] > **NEVER kill a process on a port unless you are 100% sure it was started by yourself in THIS project (e.g., a leaked process from a previous task). Killing external software's ports will lead to project failure.**

---

### 7. Verify (MANDATORY)

**You MUST run the feature's verification_command before marking as passing:**

1. Get the verification command:

   ```sql
   SELECT verification_command FROM features WHERE id = X
   ```

2. Run the exact command (e.g., `bun x playwright test --grep "feature"`):

   ```bash
   # Run the verification_command from the database
   ```

3. Confirm the test PASSES (not just runs).

> **Web:** Also use `chrome-devtools` MCP to check console for errors.
> **NO SHORTCUT:** Do not mark as passing based on visual inspection alone.

---

### 8. Commit Changes

**Before committing, confirm:**

- [ ] Ran the feature's `verification_command`
- [ ] Verification test PASSED (not just "ran")
- [ ] **Entry-Point Audit**: For server/CLI apps, verified that `main.go`/`index.js`/`main.rs` correctly imports and wires the new logic. If it's a placeholder, the feature is NOT complete.
- [ ] Checked browser console for errors (web projects)
- [ ] Checked/Updated Memory (`auto db knowledge list/set`)

```bash
git add . && git commit -m "Implement [feature]"
```

> [!NOTE] > **DO NOT call mark-pass.** The supervisor will independently verify and mark the feature as passing after this session ends.

> [!CAUTION] > **ISOLATION RULE: STOP NOW.** You have completed ONE feature. Do NOT look at the next feature. Do NOT run another SQL query.
>
> **REASON**: Continuing multiple features in one session causes context bloat and webhook spam. The supervisor needs to reset for the next iteration.

---

### 9. Signal Completion & Exit

> [!IMPORTANT] > **THIS IS THE FINAL STEP.** You MUST exit immediately. Failure to stop after one feature is a violation of protocol.

```bash
echo "CONTINUE" > .opencode-signal
```

**Output this exact text on its own line, then end your response:**

`===SESSION_COMPLETE===`

---

## Updating Plans (If Needed)

If you discover that `.conductor/` plans are outdated or incorrect:

1. **Minor updates**: Edit the plan.md directly to fix task descriptions or add missing steps.
2. **Major restructure**: If the architecture changed significantly, regenerate the plan:

   ```bash
   # Check current track
   ls .conductor/tracks/

   # Edit the plan directly
   vim .conductor/tracks/[track-name]/plan.md
   ```

> [!TIP]
> Plans should evolve with the project. If you find yourself working around the plan instead of following it, update the plan.

---

## Help Index

- **Web/JS Development**: `modules/javascript.md`
- **Go Development**: `modules/go.md`
- **Rust/CLI Development**: `modules/rust.md`
- **Testing Strategy**: `modules/testing.md`
- **Stuck Recovery**: `modules/recovery.md`
- **MCP Usage Guide**: `core/mcp_guide.md`
- **Database Operations**: `core/database.md`
- **Async Communication**: `core/communication.md`
- **Quick Orientation**: `example vibe`
- **Conductor Tracks**: `example tracks`
