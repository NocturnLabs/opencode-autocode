# Example: SQL queries for feature inspection

# List all features:

opencode-autocode db query "SELECT id, description, passes FROM features"

# Count passing/failing:

opencode-autocode db query "SELECT passes, COUNT(\*) FROM features GROUP BY passes"

# Get next feature to work on:

opencode-autocode db query "SELECT id, description FROM features WHERE passes = 0 LIMIT 1"

# Features with weak verification:

opencode-autocode db query "SELECT id, description FROM features WHERE verification_command LIKE '%cargo build%'"
