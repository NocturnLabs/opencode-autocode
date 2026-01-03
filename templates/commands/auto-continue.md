# Autonomous Work Session

{{INCLUDE core/identity.md}}

---

## Implementation Guidelines

This is a fallback template. Normally, the supervisor injects feature-specific context.

### If You're Here

1. Read `app_spec.md` to understand the project
2. Query database for features: `opencode-autocode db stats`
3. Implement ONE pending feature
4. Output `===SESSION_COMPLETE===` when done

### Rules

- Implement **one feature** per session
- Do NOT run git commands (supervisor handles commits)
- Do NOT call mark-pass (supervisor handles verification)
- Output `===SESSION_COMPLETE===` to signal completion

### Help

- **Orientation**: `opencode-autocode example vibe`
- **Database**: `opencode-autocode db stats`
- **Modules**: `templates/modules/`
