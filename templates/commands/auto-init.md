# Autonomous Initialization Session

This is the FIRST session in a long-running autonomous development process.
The goal is to set up the foundation for all future coding sessions.

**AUTONOMOUS MODE: No user input required. Work → Signal → End.**

---

### SECURITY CONSTRAINTS (MANDATORY - READ FIRST!)

**Before executing ANY commands, you MUST:**

1. Read `.autocode/security-allowlist.json` if it exists
2. Check the `blocked_patterns` array for commands you must NEVER run
3. Only use commands listed in `allowed_commands` categories

**BLOCKED PATTERNS ARE ABSOLUTE:**

- If a command matches ANY pattern in `blocked_patterns`, DO NOT RUN IT
- No exceptions, even if it seems necessary for the task
- If you need a blocked command, document the blocker and move on

Example: If `"cargo build"` is in blocked_patterns, you may NOT run `cargo build`,
`cargo build --release`, or any variation. Find an alternative approach.

---

### FIRST: Read the Project Specification

Start by reading `app_spec.md` in your working directory. This file contains
the complete specification for what you need to build. Read it carefully
before proceeding. Understand:

- What the project does
- What technology stack is involved
- What features need to be built
- What success criteria must be met

**IMPORTANT**: If you need to refine, clarify, or update the specification during
your work, **modify `app_spec.md` directly**. Do NOT create separate files like
`refined_specification.xml` or `updated_spec.md`. The single source of truth for
the project specification is always `app_spec.md`.

---

### CRITICAL FIRST TASK: Populate Features in Database

Based on `.autocode/app_spec.md`, insert features into the database using the CLI.

**Use the CLI to insert features:**

```bash
opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Brief description of what this test verifies', 0, 'npm test -- --grep \"feature-name\"')"
```

**Also insert the steps for each feature:**

```bash
opencode-autocode db exec "INSERT INTO feature_steps (feature_id, step_order, step_text) VALUES (1, 1, 'Step 1: Navigate to relevant page or run command')"
```

**Database Schema (already created in `.autocode/progress.db`):**

- `features` table: `id`, `category`, `description`, `passes` (0/1), `verification_command`, `created_at`, `updated_at`
- `feature_steps` table: `feature_id`, `step_order`, `step_text`

**Category Options:** "functional", "style", "integration", "performance"

**Requirements:**

- Insert comprehensive features covering all requirements in the spec
- Both "functional" and "style" categories (if applicable)
- Mix of narrow features (2-5 steps) and comprehensive features (10+ steps)
- Order by priority: fundamental features first
- ALL features start with `passes = 0`
- Cover every feature in the spec exhaustively
- **Add `verification_command` where possible** for automated regression testing

**CRITICAL INSTRUCTION:**
IT IS CATASTROPHIC TO DELETE OR UPDATE feature descriptions in future sessions.
Features can ONLY be marked as passing: `UPDATE features SET passes = 1 WHERE id = X`
Never delete features, never edit descriptions. This ensures no functionality is missed.

**E2E TESTING REQUIREMENTS:**

1. **Detect project type from `app_spec.md`:**

   - If the project has a frontend (web, PWA, SPA), it is a **web project**
   - CLI tools, backend APIs, and libraries are **non-web projects**

2. **For web projects:**

   - You MUST scaffold an E2E testing framework (Playwright recommended)
   - Run: `npm init playwright@latest` or equivalent for the stack
   - Create `tests/e2e/` directory for E2E test files
   - `verification_command` for each feature MUST invoke E2E tests
   - Unit tests are NOT sufficient for feature verification

3. **For non-web projects:**
   - Standard integration/unit tests are acceptable for `verification_command`
   - E2E framework is optional

---

### SECOND TASK: Create Conductor Context

Create the `.conductor/` directory with context files that inform all future sessions:

**Create `.conductor/product.md`:**

```markdown
# Product Context

## Target Users

- [Primary user persona from app_spec.md]

## Product Goals

1. [Goal 1 from spec]
2. [Goal 2 from spec]

## High-Level Features

- [Feature categories from spec]

## Success Criteria

- [From spec's success criteria]
```

**Create `.conductor/tech_stack.md`:**

```markdown
# Tech Stack

## Language/Runtime

- **Primary**: [From app_spec.md]

## Frameworks

- [Frontend/Backend frameworks from spec]

## Database

- [Database choice from spec]
```

**Create `.conductor/workflow.md`:**

```markdown
# Workflow Preferences

## Testing Strategy

- [Based on tech stack]

## Code Style

- [Based on tech stack defaults]
```

Also create the `tracks/` directory for per-feature planning:

```bash
mkdir -p .conductor tracks
```

---

### THIRD TASK: Create init.sh

Create a script called `init.sh` that future agents can use to quickly
set up and run the development environment. The script should:

1. Install any required dependencies
2. Start any necessary servers or services
3. Print helpful information about how to access the running application

Base the script on the technology stack specified in `app_spec.md`.
Make the script as portable and robust as possible.

---

### FOURTH TASK: Create .gitignore

Before initializing git, create a `.gitignore` file with common patterns:

```gitignore
# OS files
.DS_Store
Thumbs.db

# IDE/Editor
.vscode/
.idea/
*.swp
*.swo

# Semantic code search (osgrep)
.osgrep/

# Common build artifacts (adjust based on tech stack)
node_modules/
dist/
build/
target/
__pycache__/
*.pyc
.env
.env.local

# Test artifacts
coverage/
.nyc_output/
playwright-report/
test-results/
```

Adjust the ignores based on the tech stack in `app_spec.md`.

---

### FIFTH TASK: Initialize Git

Create a git repository and make your first commit with:

- .gitignore (configured for the project)
- .autocode/features.json (complete with all features)
- .conductor/ (context files)
- init.sh (environment setup script)
- README.md (project overview and setup instructions)

Commit message: "Initial setup: .autocode/features.json, conductor context, and project structure"

---

### SIXTH TASK: Create Project Structure

Set up the basic project structure based on what's specified in `app_spec.md`.
This includes directories for source code, tests, and any other components
mentioned in the spec.

---

### MCP USAGE GUIDELINES

When you need information, use the MCPs available to you:

1. **Code Search** - Use semantic search (e.g., osgrep) or grep for code patterns

   - Efficient pattern matching with minimal context window usage
   - Use for finding code patterns, function definitions, usages

2. **Local Knowledge** - If available, check knowledge bases for prior solutions
   (Note: This is supplemental knowledge only, not authoritative)

3. **Documentation** - Look up library/framework documentation (e.g., deepwiki)

4. **Web Search** - Search the web when local knowledge is insufficient

5. **Structured Reasoning** - Use reasoning tools for complex problem decomposition

---

### BEFORE COMPLEX DECISIONS

Use Sequential Thinking to structure your reasoning:

- Define the problem scope
- Research patterns (check available MCPs)
- Analyze options
- Document your reasoning chain

---

### OPTIONAL: Start Implementation

If you have time remaining in this session, you may begin implementing
the highest-priority features. Query the database for incomplete features:

```sql
SELECT id, description FROM features WHERE passes = 0 ORDER BY id LIMIT 1;
```

- Work on ONE feature at a time
- Test thoroughly before marking as passing: `UPDATE features SET passes = 1 WHERE id = X`
- Commit your progress before session ends

---

### ENDING THIS SESSION (CRITICAL!)

Before your context fills up:

1. Commit all work with descriptive messages
2. Ensure all features are inserted in the database (query to verify)
3. Leave the environment in a clean, working state

**THEN signal for continuation:**

```bash
echo "CONTINUE" > .opencode-signal
```

And output this exact message:

```
===SESSION_COMPLETE===
Ready for next iteration.
```

This signals the runner script to start a new session automatically.

**DO NOT wait for user input. DO NOT ask any questions. Just signal and end.**

---

**Remember:** You have unlimited time across many sessions. Focus on
quality over speed. Production-ready is the goal.

**AUTONOMOUS MODE:** No user interaction. Work → Commit → Signal → End.
