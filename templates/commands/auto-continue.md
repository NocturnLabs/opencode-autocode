# Autonomous Coding Session (Continuation)

Continuing work on a long-running autonomous development task.
This is a FRESH context window—no memory of previous sessions exists.

**AUTONOMOUS MODE: Work until done, then signal for continuation.**

---

### SECURITY CONSTRAINTS (MANDATORY - READ FIRST!)

**Before executing ANY commands, you MUST:**

1. Read `scripts/security-allowlist.json` if it exists
2. Check the `blocked_patterns` array for commands you must NEVER run
3. Only use commands listed in `allowed_commands` categories

**BLOCKED PATTERNS ARE ABSOLUTE:**

- If a command matches ANY pattern in `blocked_patterns`, DO NOT RUN IT
- No exceptions, even if it seems necessary for the task
- If you need a blocked command, document the blocker and move on

Example: If `"cargo build"` is in blocked_patterns, you may NOT run `cargo build`,
`cargo build --release`, or any variation. Find an alternative approach.

---

### STEP 1: GET YOUR BEARINGS (MANDATORY)

Start by orienting yourself. Examine the project structure:

1. **List files** to understand project structure
2. **Read app_spec.md** to understand what you're building
3. **Read feature_list.json** to see all work and current progress
4. **Check opencode-progress.txt** for notes from previous sessions
5. **Review git history** to see what's been done recently
6. **Count remaining work** - how many tests are still failing?
7. **Detect project type** - check `app_spec.md` for frontend/web components

Understanding `app_spec.md` is critical - it contains the full requirements
for the application you're building. **If you need to refine or update the spec,
modify `app_spec.md` directly. Do NOT create separate files like `refined_specification.xml`.**

**WEB PROJECT DETECTION:**
If `app_spec.md` contains frontend technology (React, Vue, HTML, HTMX, etc.) or mentions
"web app", "PWA", "SPA", or "frontend", this is a **web project**. For web projects:

- You MUST use `chrome-devtools` MCP for interactive verification
- Check browser console for errors after implementing features
- E2E tests (Playwright) should be used for `verification_command`

---

### STEP 2: CHECK COMPLETION STATUS

Check if all tests are passing. Use code search for efficient searching:

```
# Use code search (e.g., osgrep MCP - smaller context window footprint)
# Or fallback to grep
```

```bash
grep -c '"passes": true' feature_list.json
grep -c '"passes": false' feature_list.json
```

**If ALL tests pass (0 remaining):**

- Go to **STEP 12: FINAL E2E VERIFICATION** before signaling completion

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

### STEP 4: FULL REGRESSION TEST (CRITICAL!)

**MANDATORY BEFORE NEW WORK:**

The previous session may have introduced bugs. Before implementing anything
new, you MUST verify that ALL existing passing features still work.

1. **Get the count of passing features:**

   ```bash
   grep -c '"passes": true' feature_list.json
   ```

2. **Run verification for EVERY feature marked as passing:**

   - If the project has automated tests, run them: `npm test`, `cargo test`, `pytest`, etc.
   - For features with `verification_command`, execute that command
   - For manual-only features, walk through the documented steps

3. **Log the results:**
   ```
   STARTUP REGRESSION CHECK: X/Y tests verified, Z failures detected
   ```

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

**CRITICAL: RETRY LIMITS AND ALTERNATIVE APPROACH PROTOCOL**

If an edit or fix fails 3 times in a row:

1. **STOP** - Do NOT try the same approach again

2. **DOCUMENT** - Write the blocker to `opencode-progress.txt`:

   ```
   BLOCKED: [feature name] - [brief reason]
   Attempted: [what you tried]
   ```

3. **RESEARCH** - Begin a comprehensive search using ALL available tools:

   - **Code Search**: Search codebase efficiently (e.g., osgrep, ripgrep, grep)
   - **Structured Reasoning**: Break down the problem systematically
   - **Documentation**: Look up official docs for the library/framework
   - **Web Search**: Search for similar issues and solutions
   - **Read the actual error messages** carefully
   - **Read related source files** to understand context
   - **Check imports and dependencies** that might be missing

4. **GENERATE ALTERNATIVE APPROACHES** - When stuck, force exploration of different solutions:

   Think through 5-7 fundamentally different ways to implement this feature:

   - What's the simplest possible approach? (even if not ideal)
   - What would a different framework/library enable?
   - What if you restructured the data model?
   - What if you changed the API contract?
   - What approach would prioritize debuggability over elegance?
   - What's an unconventional solution you'd normally dismiss?

   Document these in `opencode-progress.txt`:

   ```
   ALTERNATIVE_APPROACHES: [feature name]
   1. [approach] - [trade-off]
   2. [approach] - [trade-off]
   ...
   Selected: [which approach to try and why]
   ```

5. **TRY A FUNDAMENTALLY DIFFERENT SOLUTION** - Pick an approach that is NOT a variation of what you tried before

6. **If still stuck after 3 different approaches** - Document everything and **move to the next feature**:
   ```
   BLOCKED: [feature name]
   Reason: [root cause if known]
   Research findings: [what you learned]
   Approaches tried: [list of different approaches]
   Recommended next step: [suggestion for future session]
   ```
   Then pick the next highest-priority feature with `"passes": false` and continue working.

Signs you are stuck (trigger alternative approach generation immediately):

- Repeating "Let me try a different approach" multiple times
- Same file edit failing with "oldString and newString must be different"
- Same compilation error appearing after multiple fix attempts
- Trying the same fix pattern with minor variations

**NEVER** get stuck in infinite retry loops. Generate alternatives FIRST, then act.

---

### STEP 7: VERIFY THE FEATURE

**CRITICAL:** Test like a real user would.

- Test through the actual interface (web, CLI, API - whatever applies)
- Don't just test in isolation - verify the whole workflow
- Check for edge cases
- Verify error handling

**FOR WEB PROJECTS (MANDATORY):**

If this is a web project (detected in Step 1), you MUST:

1. **Use `chrome-devtools` MCP** to open the application in a browser
2. **Navigate to the feature** you just implemented
3. **Check the browser console** (`list_console_messages`) for errors or warnings
4. **Interact with the feature** (click buttons, fill forms, etc.)
5. **Take a screenshot** to verify the UI renders correctly

If there are ANY console errors, the feature does NOT pass. Fix them first.

---

### STEP 7.5: REGRESSION CHECKPOINT (MANDATORY!)

**Before marking a new feature as passing, verify you haven't broken anything.**

This checkpoint runs after EVERY feature implementation to ensure long-running
projects stay stable throughout development.

1. **Get the list of all currently passing features:**

   ```bash
   grep -c '"passes": true' feature_list.json
   ```

2. **Run verification for EACH passing feature:**

   - Execute automated tests if available
   - Run any `verification_command` fields
   - For complex features, test the critical path

3. **If ANY regression is detected:**

   - Immediately mark that feature as `"passes": false`
   - Document in `opencode-progress.txt`:
     ```
     REGRESSION DETECTED: [regressed feature name]
     Caused by: [current feature being implemented]
     Symptoms: [what broke]
     ```
   - Fix the regression BEFORE continuing with new work
   - Only after fixing, mark BOTH features as passing

4. **Log the checkpoint results:**
   ```
   REGRESSION CHECKPOINT: X/Y tests still passing after [feature name]
   ```

**Why every feature?** This autonomous agent is designed for long-running projects.
Catching regressions immediately is cheaper than debugging cascading failures later.

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

### STEP 12: FINAL E2E VERIFICATION (BEFORE COMPLETION)

**When all features show `"passes": true`, do NOT immediately signal completion.**

Perform comprehensive end-to-end verification:

1. **Run the full automated test suite:**

   ```bash
   # Run whatever test command is appropriate for the project
   npm test || cargo test || pytest || go test ./...
   ```

2. **Start the application** and verify it runs without errors:

   - Check for startup errors
   - Verify no console errors or warnings
   - Confirm the main UI/CLI loads correctly

3. **Execute ALL verification_commands** from feature_list.json:

   ```bash
   # For each feature that has a verification_command, run it
   ```

4. **Walk through the complete user journey:**

   - Test core workflows end-to-end as a real user would
   - For web apps: Click through all major features
   - For CLIs: Run common command sequences
   - For APIs: Test key endpoints

5. **Check for hidden issues:**

   - TypeScript errors: `npm run typecheck` or `tsc --noEmit`
   - Lint errors: `npm run lint` or equivalent
   - Browser console errors (for web apps)
   - Runtime warnings or deprecation notices

6. **Log the final verification:**
   ```
   FINAL E2E VERIFICATION:
   - Automated tests: PASS/FAIL
   - Application startup: PASS/FAIL
   - Manual walkthrough: PASS/FAIL
   - No console errors: PASS/FAIL
   ```

**If ANY issues are found:**

- Mark the affected feature(s) as `"passes": false` in feature_list.json
- Document the issues in `opencode-progress.txt`
- Continue to STEP 5 (work on fixing the issues)
- Do NOT signal PROJECT_COMPLETE

**Only if EVERYTHING passes:**

- Write the completion signal:
  ```bash
  echo "COMPLETE" > .opencode-signal
  ```
- Output:
  ```
  ===PROJECT_COMPLETE===
  All features verified end-to-end.
  ```
- Exit gracefully

---

## MCP USAGE (PRIORITY ORDER)

When you need information, use the MCPs available to you:

1. **Code Search** - Use semantic search (e.g., osgrep) or grep for code patterns

   - Efficient pattern matching with minimal context window usage
   - Use for finding code patterns, function definitions, usages

2. **Local Knowledge** - Quick check for relevant past solutions
   (Note: Supplemental knowledge only, not authoritative)

3. **Documentation** - Official documentation lookup (e.g., deepwiki)

4. **Web Search** - Broader web search for solutions and patterns

5. **Structured Reasoning** - For complex reasoning tasks:
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
