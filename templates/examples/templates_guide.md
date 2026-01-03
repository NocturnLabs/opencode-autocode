# Manage Project Templates

The `templates` command allows you to list and use pre-defined project structures for scaffolding.

## 1. List Available Templates

```bash
opencode-autocode templates list
```

## 2. Use a Template

To scaffold a new project from a template:

```bash
# In an empty directory
opencode-autocode templates use web-app-fullstack
```

## 3. Customizing Templates

Templates are stored in the binary but can be overridden by placing files in your local `.opencode/` directory after scaffolding.

## 4. Agent Rule for Templates

When an agent suggests using a template, it should first run `templates list` to confirm the exact identifier, then run `templates use` to set up the foundation.
