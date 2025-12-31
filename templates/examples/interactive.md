# Interactive Spec Generation Guide

The `opencode-autocode -i` command launches a TUI for generating project specs through iterative feedback.

## 1. Generation Modes

- **Manual**: You write the Markdown spec directly in your editor.
- **AI Idea**: You provide a one-sentence idea, and the AI generates a full `app_spec.md`.

## 2. Validation Loop

After generation, you will enter a validation loop with several options:

1. **Accept**: Use the generated spec as-is.
2. **Edit**: Open the spec in your editor for manual tweaks.
3. **Refine**: Provide a prompt to the AI to modify specific parts of the spec.
4. **Regenerate**: Start over with the same idea but different parameters.

## 3. Agent Navigation

When running in autonomous mode, the agent uses subagents (if enabled) to parallelize spec validation.

**AGENT PROMPT**: If the spec is missing a "Success Criteria" or "Testing Strategy" section, ALWAYS use the **Refine** action to have the AI add it before accepting.

## 4. Final Scaffolding

Once accepted, the tool will:

1. Write `app_spec.md`
2. Initialize `.autocode/progress.db`
3. Generate `.opencode/command/` templates
4. Create the `opencode.json` configuration
