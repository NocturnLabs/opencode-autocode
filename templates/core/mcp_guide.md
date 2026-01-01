# MCP Usage Guide

When you need information, use the available MCPs in this priority order:

## 1. Code Search (osgrep / ripgrep)

Use semantic search for efficient pattern matching with minimal context window usage.

```bash
# Find code patterns, function definitions, usages
grep -rn "pattern" src/
```

## 2. Chrome DevTools (for web projects)

**MANDATORY for web projects.** Use to:

1. Open the application in a browser
2. Navigate to features you implemented
3. Check browser console (`list_console_messages`) for errors
4. Interact with elements (click, fill forms)
5. Take screenshots to verify UI

If there are ANY console errors, the feature does NOT pass.

## 3. Documentation (deepwiki)

Look up official documentation for libraries and frameworks.

## 4. Web Search

Search for solutions, patterns, and best practices when local knowledge is insufficient.

## 6. Tool Calling Protocol

**CRITICAL**: All tool calls MUST use standard markdown code blocks. DO NOT use pipes (`|`) or other custom syntaxes.

**Correct Format:**

```bash
# Example: Check port status
ss -tulpn | grep :3000
```

**Incorrect Format:**
| Bash ss -tulpn | grep :3000
