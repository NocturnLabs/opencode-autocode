# Example: SQL Queries for Feature Inspection

Use these queries to understand the current state of the project.

## 1. Feature Progress

```bash
# Get summary stats
opencode-forger db stats

# Count by category
opencode-forger db query "SELECT category, count(*) FROM features GROUP BY category"

# List failing features
opencode-forger db query "SELECT id, description FROM features WHERE passes = 0"

# Get next feature for implementation
opencode-forger db next-feature
```

## 2. Table Discovery

```bash
# List all tables
opencode-forger db tables

# Show schema for features table
opencode-forger db schema features
```

## 3. Manual Overrides

```bash
# Mark a specific feature as passing
opencode-forger db mark-pass 5

# Reset a feature (mark as failing)
opencode-forger db exec "UPDATE features SET passes = 0 WHERE id = 12"
```

## 4. Complex Filtering

```bash
# Search for features by keyword
opencode-forger db query "SELECT * FROM features WHERE description LIKE '%auth%'"

# View features with verification commands
opencode-forger db query "SELECT id, description, verification_command FROM features"
```
