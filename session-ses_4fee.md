# New session - 2025-12-09T03:10:53.002Z

**Session ID:** ses_4fee8d9b5ffeOU0aaE5xlvxgnZ
**Created:** 12/8/2025, 7:10:53 PM
**Updated:** 12/8/2025, 7:20:41 PM

---

## User

## YOUR ROLE - INITIALIZER AGENT (Session 1 of Many)

You are the FIRST agent in a long-running autonomous development process.
Your job is to set up the foundation for all future coding agents.

**CRITICAL: This is an AUTONOMOUS session. No user input required. Work → Signal → End.**

---

### FIRST: Read the Project Specification

Start by reading `app_spec.txt` in your working directory. This file contains
the complete specification for what you need to build. Read it carefully
before proceeding. Understand:

- What the project does
- What technology stack is involved
- What features need to be built
- What success criteria must be met

---

### CRITICAL FIRST TASK: Create feature_list.json

Based on `app_spec.txt`, create a file called `feature_list.json` with detailed
end-to-end test cases appropriate for the technology stack specified.

**Format:**

```json
[
  {
    "category": "functional",
    "description": "Brief description of the feature and what this test verifies",
    "steps": [
      "Step 1: Navigate to relevant page or run command",
      "Step 2: Perform action",
      "Step 3: Verify expected result"
    ],
    "passes": false
  },
  {
    "category": "style",
    "description": "Brief description of UI/UX requirement",
    "steps": [
      "Step 1: Navigate to page",
      "Step 2: Take screenshot or inspect",
      "Step 3: Verify visual requirements"
    ],
    "passes": false
  }
]
```

**Requirements for feature_list.json:**

- Include comprehensive tests covering all features in the spec
- Both "functional" and "style" categories (if applicable to the project)
- Mix of narrow tests (2-5 steps) and comprehensive tests (10+ steps)
- Order features by priority: fundamental features first
- ALL tests start with `"passes": false`
- Cover every feature in the spec exhaustively

**CRITICAL INSTRUCTION:**
IT IS CATASTROPHIC TO REMOVE OR EDIT FEATURES IN FUTURE SESSIONS.
Features can ONLY be marked as passing (change `"passes": false` to `"passes": true`).
Never remove features, never edit descriptions, never modify testing steps.
This ensures no functionality is missed.

---

### SECOND TASK: Create init.sh

Create a script called `init.sh` that future agents can use to quickly
set up and run the development environment. The script should:

1. Install any required dependencies
2. Start any necessary servers or services
3. Print helpful information about how to access the running application

Base the script on the technology stack specified in `app_spec.txt`.
Make the script as portable and robust as possible.

---

### THIRD TASK: Initialize Git

Create a git repository and make your first commit with:

- feature_list.json (complete with all features)
- init.sh (environment setup script)
- README.md (project overview and setup instructions)

Commit message: "Initial setup: feature_list.json, init.sh, and project structure"

---

### FOURTH TASK: Create Project Structure

Set up the basic project structure based on what's specified in `app_spec.txt`.
This includes directories for source code, tests, and any other components
mentioned in the spec.

---

### MCP USAGE GUIDELINES

When you need information, use MCPs in this order:

1. **chat-history** - Check for similar problems/solutions you've seen before
   (Note: This is supplemental knowledge only, not authoritative)

2. **deepwiki** - Look up library/framework documentation

3. **perplexica** - Web search when local knowledge is insufficient

4. **sequential-thinking** - Use for complex problem decomposition

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
the highest-priority features from feature_list.json. Remember:

- Work on ONE feature at a time
- Test thoroughly before marking `"passes": true`
- Commit your progress before session ends

---

### ENDING THIS SESSION (CRITICAL!)

Before your context fills up:

1. Commit all work with descriptive messages
2. Create or update `opencode-progress.txt` with a summary of what you accomplished
3. Ensure feature_list.json is complete and saved
4. Leave the environment in a clean, working state

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

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: write
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

===SESSION_COMPLETE===
Ready for next iteration.

---

## User

## YOUR ROLE - CODING AGENT

You are continuing work on a long-running autonomous development task.
This is a FRESH context window - you have no memory of previous sessions.

**CRITICAL: This is an AUTONOMOUS session. You work until done, then signal for continuation.**

---

### STEP 1: GET YOUR BEARINGS (MANDATORY)

Start by orienting yourself. Examine the project structure:

1. **List files** to understand project structure
2. **Read app_spec.txt** to understand what you're building
3. **Read feature_list.json** to see all work and current progress
4. **Check opencode-progress.txt** for notes from previous sessions
5. **Review git history** to see what's been done recently
6. **Count remaining work** - how many tests are still failing?

Understanding the `app_spec.txt` is critical - it contains the full requirements
for the application you're building.

---

### STEP 2: CHECK COMPLETION STATUS

Check if all tests are passing:

```bash
grep -c '"passes": true' feature_list.json
grep -c '"passes": false' feature_list.json
```

**If ALL tests pass (0 remaining):**

- Write `===PROJECT_COMPLETE===` to signal completion
- Exit gracefully

**If tests remain:**

- Continue to Step 3

---

### STEP 3: START DEVELOPMENT ENVIRONMENT (IF NOT RUNNING)

If `init.sh` exists, run it to set up the environment:

```bash
chmod +x init.sh
./init.sh
```

Otherwise, start any required servers or services manually and document the process.

---

### STEP 4: VERIFICATION TEST (CRITICAL!)

**MANDATORY BEFORE NEW WORK:**

The previous session may have introduced bugs. Before implementing anything
new, you MUST verify that existing passing features still work.

Run 1-2 of the feature tests marked as `"passes": true` that are most core
to the application's functionality.

**If you find ANY issues:**

- Mark that feature as `"passes": false` immediately
- Add issues to a list
- Fix all issues BEFORE moving to new features
- This includes:
  - Functional bugs
  - UI/UX issues (if applicable)
  - Console errors
  - Broken tests
  - Performance problems

---

### STEP 5: CHOOSE ONE FEATURE TO IMPLEMENT

Look at feature_list.json and find the highest-priority feature with `"passes": false`.

Focus on completing one feature perfectly in this session before moving on.
It's okay if you only complete one feature - there will be more sessions.

---

### STEP 6: IMPLEMENT THE FEATURE

Implement the chosen feature thoroughly:

1. Write the code
2. Test manually or with automated tests
3. Fix any issues discovered
4. Verify the feature works end-to-end

---

### STEP 7: VERIFY THE FEATURE

**CRITICAL:** Test like a real user would.

- Test through the actual interface (web, CLI, API - whatever applies)
- Don't just test in isolation - verify the whole workflow
- Check for edge cases
- Verify error handling

---

### STEP 8: UPDATE feature_list.json (CAREFULLY!)

**YOU CAN ONLY MODIFY ONE FIELD: "passes"**

After thorough verification, change:

```json
"passes": false
```

to:

```json
"passes": true
```

**NEVER:**

- Remove tests
- Edit test descriptions
- Modify test steps
- Combine or consolidate tests
- Reorder tests

**ONLY CHANGE "passes" FIELD AFTER THOROUGH VERIFICATION.**

---

### STEP 9: COMMIT YOUR PROGRESS

Make a descriptive git commit:

```bash
git add .
git commit -m "Implement [feature name] - verified end-to-end

- Added [specific changes]
- Tested [how you tested]
- Updated feature_list.json: marked test #X as passing
"
```

---

### STEP 10: UPDATE PROGRESS NOTES

Update `opencode-progress.txt` with:

- What you accomplished this session
- Which test(s) you completed
- Any issues discovered or fixed
- What should be worked on next
- Current completion status (e.g., "45/200 tests passing")

---

### STEP 11: SIGNAL CONTINUATION (CRITICAL!)

**After completing Steps 1-10, you MUST signal that the loop should continue.**

Write the continuation signal to a file:

```bash
echo "CONTINUE" > .opencode-signal
```

Then output this exact message:

```
===SESSION_COMPLETE===
Ready for next iteration.
```

This signals the runner script to start a new session automatically.

**DO NOT wait for user input. DO NOT ask any questions. Just signal and end.**

---

## MCP USAGE (PRIORITY ORDER)

When you need information, use MCPs in this order:

1. **chat-history** - Quick check for relevant past solutions
   (Note: Supplemental knowledge only, not authoritative)

2. **deepwiki** - Official documentation lookup

3. **perplexica** - Broader web search for solutions and patterns

4. **sequential-thinking** - For complex reasoning tasks:
   - Breaking down difficult problems
   - Planning refactors
   - Debugging complex issues
   - Making architectural decisions

---

## IMPORTANT REMINDERS

**Your Goal:** Production-quality application with all tests passing

**This Session's Goal:** Complete at least one feature perfectly

**Priority:** Fix broken tests before implementing new features

**Quality Bar:**

- All features work correctly
- Code is clean and maintainable
- Tests pass reliably
- Documentation is updated

**You have unlimited time.** Take as long as needed to get it right.
The most important thing is that you leave the codebase in a clean state.

**AUTONOMOUS MODE:** No user interaction. Work → Commit → Signal → End.

---

Begin by running Step 1 (Get Your Bearings).

---

## Assistant

```
Tool: list
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: read
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: edit
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

```
Tool: bash
```

---

## Assistant

===SESSION_COMPLETE===
Ready for next iteration.

---

