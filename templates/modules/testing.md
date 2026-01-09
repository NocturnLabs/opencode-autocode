# Testing Module

Read this module when you need to verify features with automated or interactive tests.

---

## Core Principle

> [!IMPORTANT] > **Unit tests passing ≠ Feature complete.**
> A feature is only complete when its functionality is verified through the application's actual entry point (server, CLI, etc.).

---

## E2E Testing (Playwright) for Web Projects

For web projects, every feature MUST have a Playwright E2E test.

### Setup

```bash
bun create playwright
```

### Test Structure

```javascript
// tests/e2e/feature-name.spec.js
import { test, expect } from "@playwright/test";

test("feature description", async ({ page }) => {
  await page.goto("/");
  await page.click("#element-id");
  await expect(page.locator(".result")).toBeVisible();
});
```

### Running Tests

```bash
bun x playwright test
bun x playwright test --headed  # See browser
bun x playwright test --debug   # Step through
```

---

## Integration Testing for Backend Services

For backend HTTP services (Go, Rust, Python), verification MUST include a live service check.

### Smoke Test Pattern

```bash
# Start server in background, verify, then stop
./bin/server & PID=$!; sleep 2
curl -sf http://localhost:8080/health && echo "OK"
kill $PID
```

### Curl-Based Endpoint Verification

```bash
# GET request
curl -sf http://localhost:8080/api/v1/resource

# POST request with JSON body
curl -sf -X POST -H "Content-Type: application/json" \
  -d '{"key": "value"}' http://localhost:8080/api/v1/resource
```

---

## Verification Command

Each feature in the database should have a `verification_command`.

> [!CAUTION] > `verification_command` MUST invoke an **E2E or Integration test**, NOT just unit tests.
> Unit tests with mocks can pass even if the application is completely broken.

**Web Example:**

```sql
INSERT INTO features (category, description, passes, verification_command)
VALUES ('functional', 'User can login', 0, 'bun x playwright test --grep "login"');
```

**Backend Example (preferred):**

```sql
INSERT INTO features (category, description, passes, verification_command)
VALUES ('functional', 'Chunk API returns valid response', 0, 'curl -sf http://localhost:8080/api/v1/chunk -H "Content-Type: application/json" -d "{\"url\": \"test\"}"');
```

---

## Interactive Verification (chrome-devtools)

In addition to automated tests, manually verify with chrome-devtools MCP:

1. `navigate_page` to the feature
2. `list_console_messages` - check for errors
3. `click` / `fill` - interact with elements
4. `take_screenshot` - visual verification

---

## Regression Testing

Before marking a new feature as passing, you MUST verify that no existing functionality was broken.

```bash
# List passing features to check for regressions
opencode-forger db list

# Run automated regression check
opencode-forger db check
```

If any regression is detected:

1. Identify the failing feature
2. Fix the regression BEFORE continuing
3. DO NOT mark the current feature as passing until all regressions are fixed.
