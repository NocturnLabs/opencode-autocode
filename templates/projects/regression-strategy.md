# Regression Testing Strategy

This document explains how regression testing works in this autonomous coding project.

## Overview

The autonomous agent runs **regression checks after every feature implementation** to ensure new code doesn't break existing functionality. This is critical for long-running projects where bugs can cascade.

## How It Works

```
┌─────────────────────────────────────────┐
│ 1. Implement new feature                │
│ 2. Verify new feature works             │
│ 3. REGRESSION CHECKPOINT ◄── mandatory  │
│    └─ Run ALL passing features          │
│    └─ If any fail: fix before moving on │
│ 4. Mark feature as passing              │
│ 5. Commit and continue                  │
└─────────────────────────────────────────┘
```

## The `verification_command` Field

Features in `feature_list.json` can include an optional `verification_command`:

```json
{
  "category": "functional",
  "description": "Users can log in with email and password",
  "steps": [
    "Navigate to /login",
    "Enter valid credentials",
    "Verify redirect to dashboard"
  ],
  "passes": false,
  "verification_command": "bun test -- --grep 'login'"
}
```

When a `verification_command` is present:

- The regression checker executes it automatically
- Exit code 0 = pass, non-zero = fail
- Failed tests are flagged for immediate fixing

## Running Regression Checks

### Automatic (during autonomous sessions)

The agent runs `./scripts/run-regression-check.sh` at:

- Session start (Step 4: Full Regression Test)
- After each feature (Step 7.5: Regression Checkpoint)

### Manual

```bash
# Run regression check on current feature_list.json
./scripts/run-regression-check.sh

# Run on a specific file
./scripts/run-regression-check.sh path/to/feature_list.json
```

### Via CLI

```bash
# Using the opencode-forger CLI
opencode-forger regression-check

# With options
opencode-forger regression-check --feature-list ./feature_list.json --verbose
```

## Regression Log Format

When regressions are detected, they are logged to `opencode-progress.txt`:

```
=== REGRESSION LOG ===
Session 5: Ran 12 passing tests, 1 regression detected
REGRESSION DETECTED: User login functionality
Caused by: Password reset feature implementation
Symptoms: Login form validation fails after password reset redirect
Fixed in: Session 5, commit abc123
```

## Best Practices

1. **Always add `verification_command` when possible** - Automated checks are faster and more reliable than manual verification

2. **Keep verification commands fast** - Regression checks run frequently; slow commands slow down the whole process

3. **Use specific test filters** - Instead of running the entire test suite, filter to the relevant tests:

   ```json
   "verification_command": "bun test -- --grep 'cart'"
   ```

4. **Check the regression log** - If you're resuming a project, review `opencode-progress.txt` for known issues

5. **Fix regressions immediately** - Don't skip the checkpoint; fixing later is always harder

## Troubleshooting

**Q: Regression check is too slow**
A: Consider using more specific `verification_command` patterns or running only critical features.

**Q: A feature has no automated tests**
A: Leave `verification_command` empty; the agent will do manual verification using the `steps` array.

**Q: Same regression keeps appearing**
A: Check if there's a fundamental architecture issue. Review the regression log in `opencode-progress.txt`.
