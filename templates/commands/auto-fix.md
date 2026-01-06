# Regression Fix Protocol

**MODE: EXECUTION (EMERGENCY)**

## 🛑 STOP AND READ

You have broken a previously passing feature. You are now in **REGRESSION FIX MODE**.
You CANNOT proceed to new features until you fix the regression you introduced.

{{dual_model_instructions}}

## The Situation

1. **Failing Feature**: `{{failing_feature}}`
2. **Error Output**:

```
{{error_message}}
```

3. **Context**: This likely happened during your recent changes for `{{current_feature}}`.

## Your Mission

1. {{explore_instructions}}
2. Analyze the error above.
3. Fix the regression in `{{failing_feature}}`.
4. Verify the fix using the project's test suite or the specific verification command: `{{verification_command}}`.
5. Ensure you do NOT break the current feature `{{current_feature}}` while fixing the regression.

## Critical Rules

- **DO NOT** modify the verification command or tests to make them pass (unless the requirement changed).
- **DO** modify the implementation code to satisfy the original requirements.
- **DO** run `opencode-autocode db check` frequently to verify your fix.

## Output

When you have fixed the regression, declare completion by updating `task.md` and notifying the user.
