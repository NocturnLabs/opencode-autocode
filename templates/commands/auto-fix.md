# Regression Fix Protocol

**MODE: EXECUTION (EMERGENCY)**

## 🛑 STOP AND READ

You have broken a previously passing feature. You are now in **REGRESSION FIX MODE**.
You CANNOT proceed to new features until you fix the regression you introduced.

## The Situation

1. **Failing Feature**: `{{failing_feature}}`
2. **Error Output**:

```
{{error_message}}
```

3. **Context**: This likely happened during your recent changes for `{{current_feature}}`.

## Your Mission

1. Analyze the error above.
2. Fix the regression in `{{failing_feature}}`.
3. Verify the fix using the project's test suite or the specific verification command: `{{verification_command}}`.
4. Ensure you do NOT break the current feature `{{current_feature}}` while fixing the regression.

## Critical Rules

- **DO NOT** modify the verification command or tests to make them pass (unless the requirement changed).
- **DO** modify the implementation code to satisfy the original requirements.
- **DO** run `opencode-autocode db check` frequently to verify your fix.

## Output

When you have fixed the regression, declare completion by updating `task.md` and notifying the user.
