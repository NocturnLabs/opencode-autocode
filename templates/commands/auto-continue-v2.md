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

### 2. Regression Check

Verify existing features still pass before new work:

4.  **Memory Check**:

    - Run `auto db knowledge list` to recall saved facts (ports, env vars, etc.).
    - If you discover new facts, SAVE THEM with `auto db knowledge set`.

5.  **Run `db check`**:

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

---

### 7. Verify (MANDATORY)

**You MUST run the feature's verification_command before marking as passing:**

1. Get the verification command:

   ```sql
   SELECT verification_command FROM features WHERE id = X
   ```

2. Run the exact command (e.g., `npx playwright test --grep "feature"`):

   ```bash
   # Run the verification_command from the database
   ```

3. Confirm the test PASSES (not just runs).

> **Web:** Also use `chrome-devtools` MCP to check console for errors.
> **NO SHORTCUT:** Do not mark as passing based on visual inspection alone.

---

### 8. Update & Commit

**Before running mark-pass, confirm:**

- [ ] Ran the feature's `verification_command`
- [ ] Verification test PASSED (not just "ran")
- [ ] Checked browser console for errors (web projects)
- [ ] Checked/Updated Memory (`auto db knowledge list/set`)

```bash
opencode-autocode db mark-pass X
git add . && git commit -m "Implement [feature]"
```

**CRITICAL: STOP HERE.** Do not start the next feature. Signal completion to reset context.

---

### 9. Signal

```bash
echo "CONTINUE" > .opencode-signal
```

`===SESSION_COMPLETE===`

---

## Help Index

- **Web/JS Development**: `modules/javascript.md`
- **Rust/CLI Development**: `modules/rust.md`
- **Testing Strategy**: `modules/testing.md`
- **Stuck Recovery**: `modules/recovery.md`
- **MCP Usage Guide**: `core/mcp_guide.md`
- **Database Operations**: `core/database.md`
- **Async Communication**: `core/communication.md`
- **Quick Orientation**: `example vibe`
- **Conductor Tracks**: `example tracks`
