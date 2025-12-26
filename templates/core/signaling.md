# Session Signaling

**To continue the loop:**

```bash
echo "CONTINUE" > .opencode-signal
```

Then output: `===SESSION_COMPLETE===`

**When ALL features pass:**

```bash
echo "COMPLETE" > .opencode-signal
```

Then output: `===PROJECT_COMPLETE===`

**DO NOT wait for user input. DO NOT ask any questions. Just signal and end.**
