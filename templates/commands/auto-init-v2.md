# Autonomous Initialization Session

{{INCLUDE core/base.md}}

---

## Initialization Workflow

This is the FIRST session. Set up the foundation for all future sessions.

### STEP 1: Read Project Specification

Read `app_spec.md` in your working directory. Understand:

- What the project does
- Technology stack
- Features to build
- Success criteria

**Detect project type** to know which modules you'll need later.

---

### STEP 2: Populate Features Database

Based on `app_spec.md`, insert features using the CLI:

```bash
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Feature description', 0, 'npm test -- --grep \"feature\"')"
```

**Requirements:**

- Cover ALL requirements from the spec exhaustively
- Mix "functional" and "style" categories
- ALL features start with `passes = 0`
- Add `verification_command` for automated regression testing
- **NEVER delete or edit feature descriptions in future sessions**

> **For web projects:** E2E tests (Playwright) are MANDATORY.
> See `templates/modules/testing.md` for setup.

---

### STEP 3: Create Conductor Context

Create `.conductor/` directory with context files:

```bash
mkdir -p .conductor tracks
```

Create `product.md`, `tech_stack.md`, `workflow.md` based on the spec.

---

### STEP 4: Create init.sh

Create a setup script for future sessions.

> **Web projects:** Include port conflict prevention.
> See `templates/modules/javascript.md` for template.

---

### STEP 5: Create .gitignore

Standard patterns for your tech stack (node_modules, target, **pycache**, etc.)

---

### STEP 6: Initialize Git

```bash
git init
git add .
git commit -m "Initial setup: features database, conductor context, project structure"
```

---

### STEP 7: Optional - Start Implementation

If time remains, query for first feature:

```sql
SELECT id, description FROM features WHERE passes = 0 ORDER BY id LIMIT 1
```

---

### STEP 8: Signal Continuation

```bash
echo "CONTINUE" > .opencode-signal
```

Output: `===SESSION_COMPLETE===`

---

## Help Index

| Situation              | Module                            |
| ---------------------- | --------------------------------- |
| Web/JavaScript project | `templates/modules/javascript.md` |
| Testing setup          | `templates/modules/testing.md`    |
| MCP tool usage         | `templates/core/mcp_guide.md`     |
