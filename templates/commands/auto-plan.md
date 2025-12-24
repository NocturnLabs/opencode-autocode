# Feature Planning Session

Before implementing a feature, create a formal specification and implementation plan.
This ensures structured, checkpointable work that survives across sessions.

**AUTONOMOUS MODE: Generate spec + plan, then proceed to implementation.**

---

### SECURITY CONSTRAINTS (MANDATORY - READ FIRST!)

**Before executing ANY commands, you MUST:**

1. Read `scripts/security-allowlist.json` if it exists
2. Check the `blocked_patterns` array for commands you must NEVER run
3. Only use commands listed in `allowed_commands` categories

---

### STEP 1: Identify Target Feature

Read `feature_list.json` and select the highest-priority feature with `"passes": false`.

Note the feature description and category for planning.

---

### STEP 2: Read Project Context

Load the context files to understand project constraints:

1. Read `.conductor/product.md` for product goals
2. Read `.conductor/tech_stack.md` for technical constraints
3. Read `.conductor/workflow.md` for team conventions

If these files don't exist, create them first using `/auto-context`.

---

### STEP 3: Create Feature Directory

Create the track directory for this feature:

```bash
mkdir -p tracks/{feature-name-slug}/
```

Use a URL-safe slug (lowercase, hyphens instead of spaces).

---

### STEP 4: Generate spec.md

Create `tracks/{feature-name}/spec.md` with the following structure:

```markdown
# Specification: {Feature Name}

## Overview

Brief description of what this feature does and why it's needed.

## Requirements

### Functional Requirements

- [ ] FR1: [Specific behavior]
- [ ] FR2: [Specific behavior]
- [ ] FR3: [Specific behavior]

### Non-Functional Requirements

- [ ] NFR1: [Performance, security, accessibility]
- [ ] NFR2: [Constraints or quality attributes]

## Acceptance Criteria

- [ ] AC1: [Given/When/Then or clear pass/fail condition]
- [ ] AC2: [Given/When/Then]
- [ ] AC3: [Given/When/Then]

## Edge Cases

- [Edge case 1: What happens if...]
- [Edge case 2: What happens when...]

## Out of Scope

- [Explicitly excluded functionality]
```

---

### STEP 5: Generate plan.md

Create `tracks/{feature-name}/plan.md` with the following structure:

```markdown
# Plan: {Feature Name}

## Phase 1: Setup

- [ ] Task 1.1: [Setup step]
  - [ ] Subtask 1.1.1: [Specific action]
  - [ ] Subtask 1.1.2: [Specific action]
- [ ] Task 1.2: [Setup step]

## Phase 2: Implementation

- [ ] Task 2.1: [Core functionality]
  - [ ] Subtask 2.1.1: [Specific action]
  - [ ] Subtask 2.1.2: [Specific action]
- [ ] Task 2.2: [Supporting functionality]

## Phase 3: Testing

- [ ] Task 3.1: Write unit tests
- [ ] Task 3.2: Write integration tests
- [ ] Task 3.3: Manual verification

## Phase 4: Polish

- [ ] Task 4.1: Error handling
- [ ] Task 4.2: Documentation
- [ ] Task 4.3: Code cleanup

## Checkpoints

- [ ] Checkpoint 1: Phase 1 complete, basic structure in place
- [ ] Checkpoint 2: Core functionality working
- [ ] Checkpoint 3: All tests passing
- [ ] Checkpoint 4: Feature complete and verified
```

---

### STEP 6: Commit Planning Artifacts

```bash
git add tracks/
git commit -m "Add spec and plan for: {feature name}"
```

---

### STEP 7: Begin Implementation

Now proceed with implementing the first unchecked task in `plan.md`.

As you complete each task:

1. Mark the task as complete: `- [x] Task description`
2. Commit progress periodically
3. Update `opencode-progress.txt` with status

---

### STEP 8: Signal Continuation

When you've made progress (or need a fresh context):

```bash
echo "CONTINUE" > .opencode-signal
```

Then output:

```
===SESSION_COMPLETE===
Ready for next iteration.
```

The next session will pick up where you left off using `plan.md`.
