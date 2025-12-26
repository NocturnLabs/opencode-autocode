# Autonomous Work Session

{{INCLUDE core/identity.md}}
{{INCLUDE core/security.md}}

---

## Workflow

### 1. Orient

- List files, read `app_spec.md`, detect project type
- Query: `opencode-autocode db query "SELECT COUNT(*) FROM features WHERE passes = 0"`
- Check `.autocode/session.log` for prior notes

> **Need guidance?** Read `templates/modules/[javascript|rust|testing].md`

---

### 2. Regression Check

Verify existing features still pass before new work.

---

### 3. Pick One Feature

```sql
SELECT id, description FROM features WHERE passes = 0 ORDER BY id LIMIT 1
```

---

### 4. Implement

Write code → Test → Fix → Verify end-to-end.

> **Stuck 3+ times?** Read `templates/modules/recovery.md`

---

### 5. Start Server (Web Projects - MANDATORY)

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

### 6. Verify

Test like a real user. Check console for errors.

> **Web:** Use `chrome-devtools` MCP. See `templates/modules/javascript.md`

---

### 7. Update & Commit

```bash
opencode-autocode db mark-pass X
git add . && git commit -m "Implement [feature]"
```

---

### 8. Signal

```bash
echo "CONTINUE" > .opencode-signal
```

`===SESSION_COMPLETE===`

---

## Help Index

| Situation   | Module                  |
| ----------- | ----------------------- |
| Web/JS      | `modules/javascript.md` |
| Rust/CLI    | `modules/rust.md`       |
| Testing     | `modules/testing.md`    |
| Stuck       | `modules/recovery.md`   |
| MCP usage   | `core/mcp_guide.md`     |
| DB ops      | `core/database.md`      |
| Async comms | `core/communication.md` |
