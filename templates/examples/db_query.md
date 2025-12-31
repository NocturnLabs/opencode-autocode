# Example: SQL Queries for Feature Inspection

Use these queries to understand the current state of the project.

## 1. Feature Progress

```bash
# Get summary stats
opencode-autocode db stats

# Count by category
opencode-autocode db query "SELECT category, count(*) FROM features GROUP BY category"

# List failing features
opencode-autocode db query "SELECT id, description FROM features WHERE passes = 0"

# Get next feature for implementation
opencode-autocode db next-feature
```

## 2. Table Discovery

```bash
# List all tables
opencode-autocode db tables

# Show schema for features table
opencode-autocode db schema features
```

## 3. Manual Overrides

```bash
# Mark a specific feature as passing
opencode-autocode db mark-pass 5

# Reset a feature (mark as failing)
opencode-autocode db exec "UPDATE features SET passes = 0 WHERE id = 12"
```

## 4. Complex Filtering

```bash
# Search for features by keyword
opencode-autocode db query "SELECT * FROM features WHERE description LIKE '%auth%'"

# View features with verification commands
opencode-autocode db query "SELECT id, description, verification_command FROM features"
```
